/**
 * Integration with external image optimizers.
 *
 * Tauri-команды:
 * - optimize_with_pngquant(files) — lossy PNG compression via pngquant
 * - optimize_with_mozjpeg(files) — lossless/near-lossless JPEG compression via mozjpeg
 * - check_optimizer(name) — проверить доступность оптимизатора
 */

use std::path::Path;
use std::process::Command;
use tauri::command;

use super::batch::BatchResult;

/// Сжать PNG через pngquant
#[command]
pub fn optimize_with_pngquant(files: Vec<String>) -> Result<BatchResult, String> {
    let mut result = BatchResult {
        total: files.len(),
        succeeded: 0,
        failed: 0,
        errors: Vec::new(),
    };

    // Проверяем доступность pngquant
    if !is_tool_available("pngquant") {
        return Err("pngquant not found. Install it first.".to_string());
    }

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

        if ext != "png" {
            result.failed += 1;
            result
                .errors
                .push(format!("Not a PNG file: {}", file_path));
            continue;
        }

        // pngquant --quality=65-80 --force --output output.png input.png
        let output_path = path.with_extension("pngquant.png");
        let status = Command::new("pngquant")
            .args([
                "--quality=65-80",
                "--force",
                &format!("--output={}", output_path.to_string_lossy()),
                file_path,
            ])
            .output()
            .map_err(|e| format!("Failed to run pngquant: {}", e))?;

        if status.status.success() {
            // Заменяем оригинал сжатым
            if let Err(e) = std::fs::rename(&output_path, path) {
                result.failed += 1;
                result
                    .errors
                    .push(format!("Failed to replace with optimized: {}", e));
                continue;
            }
            result.succeeded += 1;
        } else {
            let stderr = String::from_utf8_lossy(&status.stderr);
            result.failed += 1;
            result
                .errors
                .push(format!("pngquant failed for {}: {}", file_path, stderr.trim()));
        }
    }

    Ok(result)
}

/// Сжать JPEG через mozjpeg (cjpeg)
#[command]
pub fn optimize_with_mozjpeg(files: Vec<String>) -> Result<BatchResult, String> {
    let mut result = BatchResult {
        total: files.len(),
        succeeded: 0,
        failed: 0,
        errors: Vec::new(),
    };

    // Проверяем доступность cjpeg (mozjpeg)
    if !is_tool_available("cjpeg") {
        return Err("mozjpeg (cjpeg) not found. Install it first.".to_string());
    }

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

        if ext != "jpg" && ext != "jpeg" {
            result.failed += 1;
            result
                .errors
                .push(format!("Not a JPEG file: {}", file_path));
            continue;
        }

        // cjpeg -quality 85 -optimize -outfile output.jpg input.jpg
        let output_path = path.with_extension("mozjpeg.jpg");
        let status = Command::new("cjpeg")
            .args([
                "-quality",
                "85",
                "-optimize",
                "-outfile",
                &output_path.to_string_lossy(),
                file_path,
            ])
            .output()
            .map_err(|e| format!("Failed to run cjpeg: {}", e))?;

        if status.status.success() {
            if let Err(e) = std::fs::rename(&output_path, path) {
                result.failed += 1;
                result
                    .errors
                    .push(format!("Failed to replace with optimized: {}", e));
                continue;
            }
            result.succeeded += 1;
        } else {
            let stderr = String::from_utf8_lossy(&status.stderr);
            result.failed += 1;
            result
                .errors
                .push(format!("cjpeg failed for {}: {}", file_path, stderr.trim()));
        }
    }

    Ok(result)
}

/// Проверить доступность внешнего инструмента
#[command]
pub fn check_optimizer(name: String) -> Result<bool, String> {
    Ok(is_tool_available(&name))
}

// --- Вспомогательные функции ---

fn is_tool_available(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}