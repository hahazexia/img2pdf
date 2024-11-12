use image::GenericImageView;
use printpdf::{
    ColorBits, ColorSpace, Image as PdfImage, ImageTransform, ImageXObject, Mm, PdfDocument, Px,
};
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

fn main() {
    let folder_path = "D:/115/115Chrome/mass/mass/zhong_mo_de_hou_gong_qi_xiang_qu/test"; // 替换为你的文件夹路径
    let output_pdf_path = "./output.pdf"; // 输出的 PDF 文件名

    // 设定统一的 PDF 页面尺寸（A4）
    let page_width = Mm(210.0); // A4宽度（毫米）
    let page_height = Mm(297.0); // A4高度（毫米）

    // 读取文件夹内容
    let mut image_files: Vec<String> = fs::read_dir(folder_path)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() {
                let file_name = path.file_name()?.to_string_lossy().into_owned();
                println!("file_name {}", file_name);
                // 检查文件名是否以数字开头并且后面有扩展名
                if let Some(stem) = file_name.split('.').next() {
                    if stem.chars().all(char::is_numeric) {
                        return Some(file_name);
                    }
                }
            }
            None
        })
        .collect();

    // 按文件名排序
    image_files.sort_by_key(|s| s.split('.').next().unwrap().parse::<u32>().unwrap_or(0));

    println!("image_files len{}", image_files.len());

    // 创建 PDF 文档
    let (document, page1, layer1) =
        PdfDocument::new("My Document", page_width, page_height, "Layer 1");

    let mut current_layer = document.get_page(page1).get_layer(layer1);

    for file_name in image_files {
        let img_path = Path::new(folder_path).join(&file_name);
        let img = image::open(img_path).expect("Failed to open image");

        // 将 DynamicImage 转换为适合 printpdf 的格式
        let (img_width, img_height) = img.dimensions();
        let img_bytes = img.to_rgba8().into_raw(); // 转换为 RGBA 格式的字节

        // 创建 ImageXObject
        let image_xobject = ImageXObject {
            width: Px(img_width as usize),
            height: Px(img_height as usize),
            color_space: ColorSpace::Rgb, // 根据需要选择颜色空间
            bits_per_component: ColorBits::Bit8,
            interpolate: true,
            image_data: img_bytes,
            image_filter: None,
            clipping_bbox: None,
            smask: None,
        };

        // 创建 PdfImage
        let pdf_image = PdfImage::from(image_xobject);

        // 计算缩放比例
        let img_aspect_ratio = img_width as f32 / img_height as f32;
        let page_aspect_ratio = page_width.0 / page_height.0;

        let (final_width, final_height) = if img_aspect_ratio > page_aspect_ratio {
            // 图片更宽，按宽度缩放
            (page_width.0, page_width.0 / img_aspect_ratio)
        } else {
            // 图片更高，按高度缩放
            (page_height.0 * img_aspect_ratio, page_height.0)
        };

        // 添加新页面
        let (page, layer) = document.add_page(Mm(page_width.0), Mm(page_height.0), "Layer 1");

        // 获取当前页面的图层引用
        current_layer = document.get_page(page).get_layer(layer);

        // 计算居中位置
        let x_offset = (page_width.0 - final_width) / 2.0;
        let y_offset = (page_height.0 - final_height) / 2.0;

        // 将图片添加到 PDF 页面
        pdf_image.add_to_layer(current_layer.clone(), ImageTransform {
            translate_x: Some(Mm(x_offset)),
            translate_y: Some(Mm(y_offset)),
            scale_x: None,
            scale_y: None,
            rotate: None,
            dpi: None,
        });
        // pdf_image.add_to_layer(current_layer, ImageTransform::default());
    }

    // 保存 PDF 文件
    let output_file = File::create(output_pdf_path).expect("Failed to create PDF file");
    let mut writer = BufWriter::new(output_file);
    document.save(&mut writer).expect("Failed to save PDF");
}
