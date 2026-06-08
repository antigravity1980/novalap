/**
 * Metadata Stripping: удаление AI-метаданных из файлов.
 *
 * Tauri-команды:
 * - strip_metadata(files) — массовое удаление метаданных
 */

use std::path::Path;
use tauri::command;

use super::batch::BatchResult;

/// Удаление всех AI-метаданных из файлов
#[command]
pub fn strip_metadata(files: Vec<String>) -> Result<BatchResult, String> {
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

        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let strip_result = match ext.as_str() {
            "png" => strip_png_metadata(path),
            "jpg" | "jpeg" => strip_jpeg_metadata(path),
            "webp" => strip_webp_metadata(path),
            _ => Err(format!("Unsupported format for stripping: {}", ext)),
        };

        match strip_result {
            Ok(_) => {
                // Синхронизируем базу данных
                super::batch::sync_file_metadata_in_db(file_path);
                result.succeeded += 1;
            }
            Err(e) => {
                result.failed += 1;
                result.errors.push(format!("Failed to strip {}: {}", file_path, e));
            }
        }
    }

    Ok(result)
}

/// Удаление tEXt/iTXt чанков из PNG
fn strip_png_metadata(path: &Path) -> Result<(), String> {
    let data = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;

    if data.len() < 8 {
        return Err("Invalid PNG file".to_string());
    }

    // Проверяем сигнатуру
    let png_sig = [137, 80, 78, 71, 13, 10, 26, 10];
    if data[..8] != png_sig {
        return Err("Not a valid PNG file".to_string());
    }

    let mut output = Vec::new();
    output.extend_from_slice(&data[..8]); // PNG signature

    let mut pos = 8;
    while pos + 12 <= data.len() {
        let chunk_len = u32::from_be_bytes([
            data[pos],
            data[pos + 1],
            data[pos + 2],
            data[pos + 3],
        ]) as usize;

        let chunk_type = &data[pos + 4..pos + 8];
        let chunk_type_str = std::str::from_utf8(chunk_type).unwrap_or("????");

        // Пропускаем текстовые чанки (tEXt, iTXt, zTXt)
        let skip = matches!(chunk_type_str, "tEXt" | "iTXt" | "zTXt");

        if !skip {
            // Копируем чанк целиком
            output.extend_from_slice(&data[pos..pos + 8 + chunk_len + 4]);
        }

        pos += 8 + chunk_len + 4;
    }

    std::fs::write(path, &output).map_err(|e| format!("Failed to write file: {}", e))
}

/// Удаление метаданных из JPEG (пересборка без маркеров APP1/APP2)
fn strip_jpeg_metadata(path: &Path) -> Result<(), String> {
    // Используем image crate для пересохранения (автоматически удаляет метаданные)
    let img = image::open(path).map_err(|e| format!("Failed to open JPEG: {}", e))?;
    img.save(path).map_err(|e| format!("Failed to save stripped JPEG: {}", e))
}

/// Удаление метаданных из WebP
fn strip_webp_metadata(path: &Path) -> Result<(), String> {
    // Пересохраняем через image crate
    let img = image::open(path).map_err(|e| format!("Failed to open WebP: {}", e))?;
    img.save(path).map_err(|e| format!("Failed to save stripped WebP: {}", e))
}