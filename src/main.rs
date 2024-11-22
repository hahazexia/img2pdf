use image::codecs::jpeg::JpegDecoder;
use image::codecs::png::PngDecoder;
use image::codecs::webp::WebPDecoder;
use image::GenericImageView as _;
use printpdf::{
    Image, ImageTransform, Mm, Px, PdfDocument, PdfConformance, CustomPdfConformance, ImageXObject, ColorBits, ColorSpace,
};
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Cursor, Read};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let folder_path = "D:/115/115Chrome/mass/mass/zhong_mo_de_hou_gong_qi_xiang_qu/test"; // 替换为你的文件夹路径
    let folder_path = "./img";
    let output_pdf_path = "./output.pdf"; // 输出的 PDF 文件名

    // 设定统一的 PDF 页面尺寸（A4）
    let mut page_width; // A4宽度（毫米）
    let mut page_height; // A4高度（毫米）

    let dpi = 200.0;

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

    let img_path = Path::new(folder_path).join((image_files.get(0)).unwrap());
    let img_dynamic = image::open(img_path)?; // 使用 image crate 打开图片
    let (img_width, img_height) = img_dynamic.dimensions(); // 获取宽度和高度
    let img_width_mm = px_to_mm(img_width as u64, dpi as u64);
    let img_height_mm = px_to_mm(img_height as u64, dpi as u64);
    page_width = img_width_mm;
    page_height = img_height_mm;
    // 创建 PDF 文档
    let (mut document, page1, layer1) =
        PdfDocument::new("My Document", Mm(img_width_mm as f32), Mm(img_height_mm as f32), "Layer 1");

    document = document.with_conformance(PdfConformance::Custom(CustomPdfConformance {
        requires_icc_profile: false,
        requires_xmp_metadata: false,
        ..Default::default()
    }));


    let mut current_layer;
    let mut count = 0;

    for file_name in image_files {
        let img_path = Path::new(folder_path).join(&file_name);

        let mut file = File::open(img_path)?;

        // 读取文件的前几个字节以判断格式
        let mut buffer = [0; 8];
        file.read_exact(&mut buffer)?;

        let img_path = Path::new(folder_path).join(&file_name);

        let mut image_file = File::open(img_path).expect("无法打开图片文件");
        let mut image_data = Vec::new();
        image_file.read_to_end(&mut image_data).expect("读取图片文件失败");

        let img_path = Path::new(folder_path).join(&file_name);
        let img_dynamic = image::open(img_path)?; // 使用 image crate 打开图片
        let (img_width, img_height) = img_dynamic.dimensions(); // 获取宽度和高度

        println!("img_width is {}, img_height is {}", img_width, img_height);

        let image_file_2 = ImageXObject {
            width: Px(img_width as usize), // 根据实际图片的宽度设置
            height: Px(img_height as usize), // 根据实际图片的高度设置
            color_space: ColorSpace::Rgb, // 根据图片的颜色空间设置
            bits_per_component: ColorBits::Bit16, // 根据图片的位深设置
            interpolate: false,
            image_data: image_data,
            smask: None,
            image_filter: None, // 目前不支持
            clipping_bbox: None, // 目前不支持
        };

        // 将 ImageXObject 转换为 Image
        let image2 = Image::from(image_file_2);



        // 根据读取的字节判断格式
        let format = match image::guess_format(&buffer) {
            Ok(format) => format,
            Err(e) => {
                eprintln!("无法判断图像格式: {}", e);
                return Err(Box::new(e));
            }
        };

        current_layer = if count > 0 {
            let img_path = Path::new(folder_path).join(&file_name);
            let img_dynamic = image::open(img_path)?; // 使用 image crate 打开图片
            let (img_width, img_height) = img_dynamic.dimensions(); // 获取宽度和高度
            let img_width_mm = px_to_mm(img_width as u64, dpi as u64);
            let img_height_mm = px_to_mm(img_height as u64, dpi as u64);
            page_width = img_width_mm;
            page_height = img_height_mm;
            let (page, layer) = document.add_page(Mm(img_width_mm as f32), Mm(img_height_mm as f32), "Layer 1");
            document.get_page(page).get_layer(layer)
        } else {
            document.get_page(page1).get_layer(layer1)
        };

        image2.add_to_layer(current_layer, ImageTransform::default());

        count += 1;

        // let img_path = Path::new(folder_path).join(&file_name);
        // let img = match format {
        //     image::ImageFormat::Png => {
        //         let mut image_file = File::open(img_path).unwrap();
        //         let decoder = PngDecoder::new(&mut image_file)?;
        //         let temp_image = Image::try_from(decoder).unwrap();
        //         temp_image
        //     }
        //     image::ImageFormat::Jpeg => {
        //         let mut image_file = File::open(img_path).unwrap();
        //         let decoder = JpegDecoder::new(&mut image_file)?;
        //         let temp_image = Image::try_from(decoder).unwrap();
        //         temp_image
        //     }
        //     image::ImageFormat::WebP => {
        //         let mut image_file = File::open(img_path).unwrap();
        //         let decoder = WebPDecoder::new(&mut image_file)?;
        //         let temp_image = Image::try_from(decoder).unwrap();
        //         temp_image
        //     }
        //     _ => {
        //         eprintln!("不支持的图像格式");
        //         return Err("不支持的图像格式".into());
        //     }
        // };

        // let img_path = Path::new(folder_path).join(&file_name);
        // let img_dynamic = image::open(img_path)?; // 使用 image crate 打开图片
        // let (img_width, img_height) = img_dynamic.dimensions(); // 获取宽度和高度
        // let img_width_mm = px_to_mm(img_width as u64, dpi as u64);
        // let img_height_mm = px_to_mm(img_height as u64, dpi as u64);

        // // 计算缩放比例
        // let scale_x = page_width / (img_width_mm);
        // let scale_y = page_height / (img_height_mm);
        // let scale = scale_x.min(scale_y); // 选择较小的比例以保持纵横比
        // println!("scale is {} scale_x is {}, scale_y is {}", scale, scale_x, scale_y);
        // // 计算居中位置
        // // let translate_x = (page_width.0 - img_width as f32 * scale) / 2.0;
        // // let translate_y = (page_height.0 - img_height as f32 * scale) / 2.0;

        // // 创建变换
        // let transform = ImageTransform {
        //     translate_x: None,
        //     translate_y: None,
        //     scale_x: Some(scale_x as f32),
        //     scale_y: Some(scale_y as f32),
        //     dpi: Some(300.0),
        //     ..Default::default()
        // };


        // // img.add_to_layer(current_layer, ImageTransform::default());
        // img.add_to_layer(current_layer, transform);
    }
    println!("count is {}", count);
    // 保存 PDF 文件
    let output_file = File::create(output_pdf_path).expect("Failed to create PDF file");
    let mut writer = BufWriter::new(output_file);
    document.save(&mut writer).expect("Failed to save PDF");
    Ok(())
}

fn px_to_mm(px: u64, dpi: u64) -> u64 {
    (px / dpi) * 25.4 as u64
}