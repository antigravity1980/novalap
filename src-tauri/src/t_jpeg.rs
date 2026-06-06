use image::RgbImage;
use image::ImageReader;
use std::fs::File;
use std::io::BufReader;
use fast_image_resize as fir;

pub fn decode_rgb8_scaled(
    file_path: &str,
    target_width: u32,
    target_height: u32,
) -> Result<(Vec<u8>, u32, u32), String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let img_reader = ImageReader::new(reader)
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    
    let img = img_reader.decode().map_err(|e| e.to_string())?;
    let rgb = img.to_rgb8();
    let (src_w, src_h) = rgb.dimensions();

    if target_width == 0 || target_height == 0 {
        return Ok((rgb.into_raw(), src_w, src_h));
    }

    if src_w <= target_width && src_h <= target_height {
        return Ok((rgb.into_raw(), src_w, src_h));
    }

    // Calculate aspect-preserving scale to fit target_width / target_height
    let scale_w = target_width as f32 / src_w as f32;
    let scale_h = target_height as f32 / src_h as f32;
    let scale = scale_w.min(scale_h);
    let dst_w = ((src_w as f32) * scale).round().max(1.0) as u32;
    let dst_h = ((src_h as f32) * scale).round().max(1.0) as u32;

    let src_image = fir::images::Image::from_vec_u8(
        src_w,
        src_h,
        rgb.into_raw(),
        fir::PixelType::U8x3,
    )
    .map_err(|e| format!("Failed to prepare RGB source image for resize: {}", e))?;

    let mut dst_image = fir::images::Image::new(dst_w, dst_h, fir::PixelType::U8x3);
    let mut resizer = fir::Resizer::new();
    let options = fir::ResizeOptions::new().resize_alg(fir::ResizeAlg::Convolution(fir::FilterType::Bilinear));

    resizer
        .resize(&src_image, &mut dst_image, &options)
        .map_err(|e| format!("Failed to resize RGB: {}", e))?;

    Ok((dst_image.into_vec(), dst_w, dst_h))
}

pub fn encode_rgb8(rgb: &RgbImage, quality: u8) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, quality);
    encoder.encode_image(rgb).map_err(|e| e.to_string())?;
    Ok(bytes)
}
