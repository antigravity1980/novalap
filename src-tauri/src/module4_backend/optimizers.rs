/**
 * Integration with external image optimizers.
 *
 * Tauri-команды:
 * - optimize_with_pngquant(files) — lossy PNG compression via pngquant
 * - optimize_with_mozjpeg(files) — lossless/near-lossless JPEG compression via mozjpeg
 * - check_optimizer(name) — проверить доступность оптимизатора
 */

use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;
use tauri::command;

use super::batch::BatchResult;

/// Поиск пути к исполняемому файлу утилиты
fn find_tool_path(name: &str) -> Option<PathBuf> {
    // 1. Проверяем в системном PATH
    if let Ok(paths) = env::var("PATH") {
        for path in env::split_paths(&paths) {
            let exe_name = if cfg!(target_os = "windows") {
                format!("{}.exe", name)
            } else {
                name.to_string()
            };
            let exe_path = path.join(&exe_name);
            if exe_path.exists() && exe_path.is_file() {
                return Some(exe_path);
            }
        }
    }

    // 2. Проверяем в папке запущенного исполняемого файла
    if let Ok(mut current_exe) = env::current_exe() {
        if current_exe.pop() {
            let exe_name = if cfg!(target_os = "windows") {
                format!("{}.exe", name)
            } else {
                name.to_string()
            };
            let exe_path = current_exe.join(&exe_name);
            if exe_path.exists() && exe_path.is_file() {
                return Some(exe_path);
            }

            // Дополнительные поддиректории
            for sub in &["resources", "third_party", "optimizers", "bin", "ffmpeg"] {
                let sub_path = current_exe.join(sub).join(&exe_name);
                if sub_path.exists() && sub_path.is_file() {
                    return Some(sub_path);
                }
            }
        }
    }

    // 3. Проверяем в текущей рабочей директории
    if let Ok(cwd) = env::current_dir() {
        let exe_name = if cfg!(target_os = "windows") {
            format!("{}.exe", name)
        } else {
            name.to_string()
        };
        let exe_path = cwd.join(&exe_name);
        if exe_path.exists() && exe_path.is_file() {
            return Some(exe_path);
        }

        for sub in &["resources", "third_party", "optimizers", "bin", "ffmpeg"] {
            let sub_path = cwd.join(sub).join(&exe_name);
            if sub_path.exists() && sub_path.is_file() {
                return Some(sub_path);
            }
        }
    }

    None
}

fn is_tool_available(name: &str) -> bool {
    find_tool_path(name).is_some()
}

/// Сжать PNG через pngquant
#[command]
pub fn optimize_with_pngquant(files: Vec<String>) -> Result<BatchResult, String> {
    let mut result = BatchResult {
        total: files.len(),
        succeeded: 0,
        failed: 0,
        errors: Vec::new(),
    };

    let tool_path = find_tool_path("pngquant")
        .ok_or_else(|| "pngquant not found. Install it first.".to_string())?;

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
        let status = Command::new(&tool_path)
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

    let tool_path = find_tool_path("cjpeg")
        .ok_or_else(|| "mozjpeg (cjpeg) not found. Install it first.".to_string())?;

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
        let status = Command::new(&tool_path)
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
    let resolved_name = if name == "pngquant" { "pngquant" } else { "cjpeg" };
    Ok(is_tool_available(resolved_name))
}