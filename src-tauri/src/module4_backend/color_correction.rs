/**
 * Batch Color Correction — массовая цветокоррекция.
 *
 * Tauri-команды:
 * - batch_color_correct(files, saturation, gamma) — изменение saturation и gamma
 */

use std::path::Path;
use tauri::command;

use super::batch::BatchResult;

/// Массовая цветокоррекция
#[command]
pub fn batch_color_correct(
    files: Vec<String>,
    saturation: f32,
    gamma: f32,
) -> Result<BatchResult, String> {
    let mut result = BatchResult {
        total: files.len(),
        succeeded: 0,
        failed: 0,
        errors: Vec::new(),
    };

    for file_path in &files {
        let path = Path::new(file_path);

        if !path.exists() {
            result.failed += 1;
            result.errors.push(format!("File not found: {}", file_path));
            continue;
        }

        let mut img = match image::open(path) {
            Ok(img) => img,
            Err(e) => {
                result.failed += 1;
                result
                    .errors
                    .push(format!("Failed to open {}: {}", file_path, e));
                continue;
            }
        };

        // Применяем гамма-коррекцию
        if (gamma - 1.0).abs() > f32::EPSILON {
            img = img.brighten((gamma.log2() * 256.0) as i32);
        }

        // Применяем насыщенность
        if (saturation - 1.0).abs() > f32::EPSILON {
            img = adjust_saturation(&img, saturation);
        }

        if let Err(e) = img.save(path) {
            result.failed += 1;
            result
                .errors
                .push(format!("Failed to save {}: {}", file_path, e));
            continue;
        }

        result.succeeded += 1;
    }

    Ok(result)
}

/// Регулировка насыщенности изображения
fn adjust_saturation(img: &image::DynamicImage, saturation: f32) -> image::DynamicImage {
    let rgb = img.to_rgba8();
    let (w, h) = rgb.dimensions();
    let mut output = image::RgbaImage::new(w, h);

    for (x, y, pixel) in rgb.enumerate_pixels() {
        let r = pixel[0] as f32;
        let g = pixel[1] as f32;
        let b = pixel[2] as f32;
        let a = pixel[3];

        // Преобразование в HSV-like: luminance + saturation adjustment
        let lum = 0.299 * r + 0.587 * g + 0.114 * b;
        let new_r = clamp_byte(lum + saturation * (r - lum));
        let new_g = clamp_byte(lum + saturation * (g - lum));
        let new_b = clamp_byte(lum + saturation * (b - lum));

        output.put_pixel(x, y, image::Rgba([new_r, new_g, new_b, a]));
    }

    image::DynamicImage::from(output)
}

fn clamp_byte(val: f32) -> u8 {
    val.round().clamp(0.0, 255.0) as u8
}