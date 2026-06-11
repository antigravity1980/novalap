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

#[allow(dead_code)]\nfn get_writable_optimizers_dir() -> PathBuf {
    // Try current_exe dir/optimizers first
    if let Ok(mut current_exe) = env::current_exe() {
        if current_exe.pop() {
            let exe_opt_dir = current_exe.join("optimizers");
            if std::fs::create_dir_all(&exe_opt_dir).is_ok() {
                let test_file = exe_opt_dir.join(".write_test");
                if std::fs::write(&test_file, "").is_ok() {
                    let _ = std::fs::remove_file(test_file);
                    return exe_opt_dir;
                }
            }
        }
    }

    // Fallback to local appdata: AppData/Local/LapAI/optimizers
    if let Some(local_data) = dirs::data_local_dir() {
        let app_data_dir = local_data.join("LapAI").join("optimizers");
        let _ = std::fs::create_dir_all(&app_data_dir);
        return app_data_dir;
    }

    // Last resort: temp directory
    std::env::temp_dir().join("LapAI").join("optimizers")
}

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

    // 2b. Проверяем в AppData/Local/LapAI/optimizers (Windows fallback)
    if let Some(local_data) = dirs::data_local_dir() {
        let exe_name = if cfg!(target_os = "windows") {
            format!("{}.exe", name)
        } else {
            name.to_string()
        };
        let app_data_path = local_data.join("LapAI").join("optimizers").join(&exe_name);
        if app_data_path.exists() && app_data_path.is_file() {
            return Some(app_data_path);
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
pub async fn optimize_with_pngquant(files: Vec<String>) -> Result<BatchResult, String> {
    let tool_path = find_tool_path("pngquant")
        .ok_or_else(|| "pngquant not found. Install it first.".to_string())?;

    let mut join_set = tokio::task::JoinSet::new();

    for file_path in files {
        let tool_path = tool_path.clone();
        join_set.spawn_blocking(move || {
            let path = Path::new(&file_path);
            if !path.exists() {
                return Err((file_path, "File not found".to_string()));
            }

            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            if ext != "png" {
                return Err((file_path, "Not a PNG file".to_string()));
            }

            let output_path = path.with_extension("pngquant.png");
            let status = Command::new(&tool_path)
                .args([
                    "--quality=65-80",
                    "--force",
                    &format!("--output={}", output_path.to_string_lossy()),
                    &file_path,
                ])
                .output();

            match status {
                Ok(out) => {
                    if out.status.success() {
                        if let Err(e) = std::fs::rename(&output_path, path) {
                            Err((file_path, format!("Failed to replace with optimized: {}", e)))
                        } else {
                            Ok(file_path)
                        }
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        Err((file_path, format!("pngquant failed: {}", stderr.trim())))
                    }
                }
                Err(e) => Err((file_path, format!("Failed to run pngquant: {}", e))),
            }
        });
    }

    let mut result = BatchResult {
        total: 0,
        succeeded: 0,
        failed: 0,
        errors: Vec::new(),
    };

    while let Some(res) = join_set.join_next().await {
        result.total += 1;
        match res {
            Ok(Ok(_path)) => {
                result.succeeded += 1;
            }
            Ok(Err((path, err))) => {
                result.failed += 1;
                result.errors.push(format!("{}: {}", path, err));
            }
            Err(e) => {
                result.failed += 1;
                result.errors.push(format!("Task panicked: {}", e));
            }
        }
    }

    Ok(result)
}

/// Сжать JPEG через mozjpeg (cjpeg)
#[command]
pub async fn optimize_with_mozjpeg(files: Vec<String>) -> Result<BatchResult, String> {
    let mut join_set = tokio::task::JoinSet::new();

    for file_path in files {
        join_set.spawn_blocking(move || {
            let path = Path::new(&file_path);
            if !path.exists() {
                return Err((file_path, "File not found".to_string()));
            }

            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            if ext != "jpg" && ext != "jpeg" {
                return Err((file_path, "Not a JPEG file".to_string()));
            }

            let output_path = path.with_extension("mozjpeg.jpg");

            // Native Rust JPEG compression
            let encode_res = (|| -> Result<(), String> {
                let img = image::ImageReader::open(path)
                    .map_err(|e| format!("Failed to open image: {}", e))?
                    .decode()
                    .map_err(|e| format!("Failed to decode image: {}", e))?;

                let out_file = std::fs::File::create(&output_path)
                    .map_err(|e| format!("Failed to create temp output file: {}", e))?;

                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(out_file, 85);
                encoder.encode_image(&img)
                    .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
                
                Ok(())
            })();

            match encode_res {
                Ok(()) => {
                    if let Err(e) = std::fs::rename(&output_path, path) {
                        let _ = std::fs::remove_file(&output_path);
                        Err((file_path, format!("Failed to replace with optimized: {}", e)))
                    } else {
                        Ok(file_path)
                    }
                }
                Err(e) => {
                    let _ = std::fs::remove_file(&output_path);
                    Err((file_path, e))
                }
            }
        });
    }

    let mut result = BatchResult {
        total: 0,
        succeeded: 0,
        failed: 0,
        errors: Vec::new(),
    };

    while let Some(res) = join_set.join_next().await {
        result.total += 1;
        match res {
            Ok(Ok(_path)) => {
                result.succeeded += 1;
            }
            Ok(Err((path, err))) => {
                result.failed += 1;
                result.errors.push(format!("{}: {}", path, err));
            }
            Err(e) => {
                result.failed += 1;
                result.errors.push(format!("Task panicked: {}", e));
            }
        }
    }

    Ok(result)
}

/// Проверить доступность внешнего инструмента
#[command]
pub fn check_optimizer(name: String) -> Result<bool, String> {
    if name == "cjpeg" {
        return Ok(true); // Built-in native JPEG compression
    }
    Ok(is_tool_available("pngquant"))
}

/// Автоматически скачать оптимизаторы cjpeg и pngquant для Windows
#[command]
pub async fn download_optimizers() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let dest_dir = get_writable_optimizers_dir();
        std::fs::create_dir_all(&dest_dir).map_err(|e| format!("Failed to create optimizers directory: {}", e))?;

        let pngquant_url = "https://raw.githubusercontent.com/imagemin/pngquant-bin/main/vendor/win/pngquant.exe";

        let client = reqwest::Client::new();

        // Helper to download a file
        async fn download_file(client: &reqwest::Client, url: &str, dest_path: &Path) -> Result<(), String> {
            let response = client.get(url).send().await.map_err(|e| format!("Request failed: {}", e))?;
            if !response.status().is_success() {
                return Err(format!("Server returned error: {}", response.status()));
            }
            let bytes = response.bytes().await.map_err(|e| format!("Failed to read response body: {}", e))?;
            std::fs::write(dest_path, bytes).map_err(|e| format!("Failed to write file to disk: {}", e))?;
            Ok(())
        }

        // Download pngquant.exe
        let pngquant_path = dest_dir.join("pngquant.exe");
        if !pngquant_path.exists() {
            download_file(&client, pngquant_url, &pngquant_path).await.map_err(|e| format!("Failed to download pngquant: {}", e))?;
        }

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}