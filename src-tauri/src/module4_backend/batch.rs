/**
 * Batch Processing: массовые трансформации файлов.
 *
 * Tauri-команды:
 * - batch_resize(files, preset) — ресайз по пресетам
 * - batch_convert(files, format, quality) — изменение формата
 * - batch_rename(files, mask, counter_start) — массовое переименование
 */

use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::command;

/// Пресет ресайза
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResizePreset {
    pub width: u32,
    pub height: u32,
    pub fit: String, // "contain", "cover", "fill", "exact"
}

/// Результат batch-операции
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// Массовый ресайз изображений
#[command]
pub fn batch_resize(files: Vec<String>, preset: ResizePreset) -> Result<BatchResult, String> {
    println!("[BatchResize] Starting batch resize for {} files with preset: Width={}, Height={}, Fit={}", files.len(), preset.width, preset.height, preset.fit);
    let mut result = BatchResult {
        total: files.len(),
        succeeded: 0,
        failed: 0,
        errors: Vec::new(),
    };

    for file_path in &files {
        let path = Path::new(file_path);

        // Проверяем существование
        if !path.exists() {
            println!("[BatchResize] Error: File not found - {}", file_path);
            result.failed += 1;
            result.errors.push(format!("File not found: {}", file_path));
            continue;
        }

        // Загружаем изображение
        let img = match image::open(path) {
            Ok(img) => {
                println!("[BatchResize] Opened: {} ({}x{})", file_path, img.width(), img.height());
                img
            }
            Err(e) => {
                println!("[BatchResize] Error: Failed to open {} - {}", file_path, e);
                result.failed += 1;
                result.errors.push(format!("Failed to open {}: {}", file_path, e));
                continue;
            }
        };

        // Вычисляем новые размеры
        let (new_width, new_height) = calculate_dimensions(
            img.width(),
            img.height(),
            preset.width,
            preset.height,
            &preset.fit,
        );
        println!("[BatchResize] Resizing {} from {}x{} to {}x{}", file_path, img.width(), img.height(), new_width, new_height);

        // Ресайзим
        let resized = img.resize_exact(
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );

        // Сохраняем (перезаписываем)
        if let Err(e) = resized.save(path) {
            println!("[BatchResize] Error: Failed to save {} - {}", file_path, e);
            result.failed += 1;
            result.errors.push(format!("Failed to save {}: {}", file_path, e));
            continue;
        }

        println!("[BatchResize] Successfully resized and saved {}", file_path);
        result.succeeded += 1;
    }

    println!("[BatchResize] Finished batch resize. Success: {}, Failed: {}", result.succeeded, result.failed);
    Ok(result)
}

/// Массовое изменение формата
#[command]
pub fn batch_convert(
    files: Vec<String>,
    target_format: String,
    quality: u8,
) -> Result<BatchResult, String> {
    let mut result = BatchResult {
        total: files.len(),
        succeeded: 0,
        failed: 0,
        errors: Vec::new(),
    };

    let quality = quality.clamp(1, 100);

    for file_path in &files {
        let path = Path::new(file_path);

        if !path.exists() {
            result.failed += 1;
            result.errors.push(format!("File not found: {}", file_path));
            continue;
        }

        let img = match image::open(path) {
            Ok(img) => img,
            Err(e) => {
                result.failed += 1;
                result.errors.push(format!("Failed to open {}: {}", file_path, e));
                continue;
            }
        };

        // Создаём новое имя файла с новым расширением
        let new_path = path.with_extension(&target_format);

        // Сохраняем с качеством
        let save_result = match target_format.to_lowercase().as_str() {
            "jpg" | "jpeg" => {
                let mut output = std::fs::File::create(&new_path)
                    .map_err(|e| format!("Failed to create file: {}", e))?;
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                    &mut output,
                    quality,
                );
                encoder.encode(
                    img.as_bytes(),
                    img.width(),
                    img.height(),
                    img.color().into(),
                )
                .map_err(|e| format!("Failed to encode JPEG: {}", e))
            }
            "png" => {
                img.save(&new_path)
                    .map_err(|e| format!("Failed to save PNG: {}", e))
            }
            "webp" => {
                // WebP через image crate (если поддерживается)
                img.save(&new_path)
                    .map_err(|e| format!("Failed to save WebP: {}", e))
            }
            "avif" => {
                img.save(&new_path)
                    .map_err(|e| format!("Failed to save AVIF: {}", e))
            }
            "bmp" => {
                img.save(&new_path)
                    .map_err(|e| format!("Failed to save BMP: {}", e))
            }
            "gif" => {
                img.save(&new_path)
                    .map_err(|e| format!("Failed to save GIF: {}", e))
            }
            _ => Err(format!("Unsupported target format: {}", target_format)),
        };

        match save_result {
            Ok(_) => {
                // Если успешно, удаляем оригинал
                let _ = std::fs::remove_file(path);
                result.succeeded += 1;
            }
            Err(e) => {
                result.failed += 1;
                result.errors.push(format!("Failed to convert {}: {}", file_path, e));
            }
        }
    }

    Ok(result)
}

/// Массовое переименование
#[command]
pub fn batch_rename(
    files: Vec<String>,
    mask: String,
    counter_start: u32,
) -> Result<BatchResult, String> {
    let mut result = BatchResult {
        total: files.len(),
        succeeded: 0,
        failed: 0,
        errors: Vec::new(),
    };

    let mut counter = counter_start;

    for file_path in &files {
        let path = Path::new(file_path);

        if !path.exists() {
            result.failed += 1;
            result.errors.push(format!("File not found: {}", file_path));
            continue;
        }

        // Определяем расширение
        let extension = path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();

        // Формируем новое имя по маске
        let new_name = mask
            .replace("{n}", &counter.to_string())
            .replace("{counter}", &counter.to_string())
            .replace("{ext}", &extension)
            .replace("{EXT}", &extension.to_uppercase());

        let new_path = path.with_file_name(&new_name);

        // Если такой файл уже существует, добавляем суффикс
        let final_path = if new_path.exists() {
            let stem = new_path.file_stem().unwrap_or_default();
            let new_name_unique = format!("{}_{}{}", stem.to_string_lossy(), counter, extension);
            path.with_file_name(&new_name_unique)
        } else {
            new_path
        };

        if let Err(e) = std::fs::rename(path, &final_path) {
            result.failed += 1;
            result.errors.push(format!("Failed to rename {}: {}", file_path, e));
        } else {
            result.succeeded += 1;
        }

        counter += 1;
    }

    Ok(result)
}

// --- Вспомогательные функции ---

/// Вычислить новые размеры с учётом fit
fn calculate_dimensions(
    orig_width: u32,
    orig_height: u32,
    target_width: u32,
    target_height: u32,
    fit: &str,
) -> (u32, u32) {
    if target_width == 0 && target_height == 0 {
        return (orig_width, orig_height);
    }
    if target_width == 0 {
        let scale = target_height as f64 / orig_height as f64;
        let new_width = (orig_width as f64 * scale).round() as u32;
        return (new_width.max(1), target_height);
    }
    if target_height == 0 {
        let scale = target_width as f64 / orig_width as f64;
        let new_height = (orig_height as f64 * scale).round() as u32;
        return (target_width, new_height.max(1));
    }

    match fit {
        "exact" => (target_width, target_height),
        "contain" => {
            let scale = f64::min(
                target_width as f64 / orig_width as f64,
                target_height as f64 / orig_height as f64,
            );
            (
                ((orig_width as f64 * scale).round() as u32).max(1),
                ((orig_height as f64 * scale).round() as u32).max(1),
            )
        }
        "cover" => {
            let scale = f64::max(
                target_width as f64 / orig_width as f64,
                target_height as f64 / orig_height as f64,
            );
            (
                ((orig_width as f64 * scale).round() as u32).max(1),
                ((orig_height as f64 * scale).round() as u32).max(1),
            )
        }
        "fill" => (target_width, target_height),
        _ => (orig_width, orig_height),
    }
}