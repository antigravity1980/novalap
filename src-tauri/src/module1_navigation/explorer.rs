/**
 * Explorer Mode: чтение файловой системы «на лету».
 *
 * Tauri-команды:
 * - list_directory(path) — список содержимого директории
 * - get_drives() — список дисков (Windows) / mount points (Linux/macOS)
 * - get_file_info(path) — детальная информация о файле
 */

extern crate imagesize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::command;

use crate::t_common;

/// Тип записи в файловой системе
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_file: bool,
    pub size: u64,
    pub modified: String,
    pub created: Option<String>,
    pub extension: Option<String>,
    pub resolution: Option<Resolution>,
    pub dir_count: Option<u32>,
    pub file_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

/// Информация о диске / mount point
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DriveInfo {
    pub name: String,
    pub path: String,
    pub total_space: u64,
    pub free_space: u64,
    pub is_removable: bool,
}

/// Список файлов и директорий по пути
#[command]
pub fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    let dir_path = Path::new(&path);

    if !dir_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    let mut entries: Vec<FileEntry> = Vec::new();
    let read_dir = fs::read_dir(dir_path).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let file_name = entry.file_name().to_string_lossy().to_string();
        let entry_path = entry.path();
        let path_str = entry_path.to_string_lossy().to_string();
        let is_dir = metadata.is_dir();
        let is_file = metadata.is_file();
        let size = metadata.len();

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| {
                let duration = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                    .map(|dt| dt.to_rfc3339())
            })
            .unwrap_or_default();

        let created = metadata
            .created()
            .ok()
            .and_then(|t| {
                let duration = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                    .map(|dt| dt.to_rfc3339())
            });

        let extension = entry_path
            .extension()
            .map(|ext| ext.to_string_lossy().to_lowercase());

        let resolution = if is_file {
            get_file_resolution(&entry_path, extension.as_deref())
        } else {
            None
        };

        let (dir_count, file_count) = if is_dir {
            if let Ok(read_subdir) = fs::read_dir(&entry_path) {
                let mut dc = 0;
                let mut fc = 0;
                for sub_entry in read_subdir {
                    if let Ok(se) = sub_entry {
                        if let Ok(meta) = se.metadata() {
                            if meta.is_dir() {
                                dc += 1;
                            } else if meta.is_file() {
                                fc += 1;
                            }
                        }
                    }
                }
                (Some(dc), Some(fc))
            } else {
                (Some(0), Some(0))
            }
        } else {
            (None, None)
        };

        entries.push(FileEntry {
            name: file_name,
            path: path_str,
            is_dir,
            is_file,
            size,
            modified,
            created,
            extension,
            resolution,
            dir_count,
            file_count,
        });
    }

    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(entries)
}

/// Получить список дисков (Windows)
#[cfg(target_os = "windows")]
#[command]
pub fn get_drives() -> Result<Vec<DriveInfo>, String> {
    let mut drives = Vec::new();
    for letter in 'A'..='Z' {
        let path = format!("{}:\\", letter);
        let path_obj = Path::new(&path);
        if path_obj.exists() {
            drives.push(DriveInfo {
                name: format!("{}:", letter),
                path,
                total_space: 0,
                free_space: 0,
                is_removable: letter == 'A' || letter == 'B',
            });
        }
    }
    Ok(drives)
}

/// Получить список mount points (Unix)
#[cfg(target_family = "unix")]
#[command]
pub fn get_drives() -> Result<Vec<DriveInfo>, String> {
    let mount_paths = vec![
        "/".to_string(),
        "/media".to_string(),
        "/mnt".to_string(),
        "/Volumes".to_string(),
        "/run/media".to_string(),
    ];
    let drives = mount_paths
        .into_iter()
        .filter(|p| Path::new(p).exists())
        .map(|p| {
            let is_rem = p.contains("media") || p.contains("Volumes") || p == "/mnt";
            DriveInfo {
                name: p.clone(),
                path: p,
                total_space: 0,
                free_space: 0,
                is_removable: is_rem,
            }
        })
        .collect();
    Ok(drives)
}

#[cfg(not(any(target_os = "windows", target_family = "unix")))]
#[command]
pub fn get_drives() -> Result<Vec<DriveInfo>, String> {
    Ok(Vec::new())
}

/// Получить информацию о файле
#[command]
pub fn get_file_entry(path: String) -> Result<FileEntry, String> {
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    let metadata = fs::metadata(&path).map_err(|e| format!("Failed to get metadata: {}", e))?;
    let is_dir = metadata.is_dir();
    let is_file = metadata.is_file();
    let size = metadata.len();
    let extension = file_path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase());
    let resolution = if is_file {
        get_file_resolution(file_path, extension.as_deref())
    } else {
        None
    };

    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| {
            let duration = t.duration_since(std::time::UNIX_EPOCH).ok()?;
            chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                .map(|dt| dt.to_rfc3339())
        })
        .unwrap_or_default();

    let name = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string_lossy().to_string());

    let (dir_count, file_count) = if is_dir {
        if let Ok(read_subdir) = fs::read_dir(file_path) {
            let mut dc = 0;
            let mut fc = 0;
            for sub_entry in read_subdir {
                if let Ok(se) = sub_entry {
                    if let Ok(meta) = se.metadata() {
                        if meta.is_dir() {
                            dc += 1;
                        } else if meta.is_file() {
                            fc += 1;
                        }
                    }
                }
            }
            (Some(dc), Some(fc))
        } else {
            (Some(0), Some(0))
        }
    } else {
        (None, None)
    };

    Ok(FileEntry {
        name,
        path: file_path.to_string_lossy().to_string(),
        is_dir,
        is_file,
        size,
        modified,
        created: None,
        extension,
        resolution,
        dir_count,
        file_count,
    })
}

/// Получить эскиз изображения в base64
#[command]
pub async fn get_explorer_thumbnail(path: String, size: u32) -> Result<String, String> {
    use base64::{Engine, engine::general_purpose};
    let path_clone = path.clone();
    tokio::task::spawn_blocking(move || {
        let p = Path::new(&path_clone);
        let is_raw = crate::t_libraw::is_tiff_path(&path_clone) || 
            p.extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .map(|ext| {
                    matches!(ext.as_str(), "cr2" | "cr3" | "nef" | "arw" | "dng" | "orf" | "rw2" | "pef" | "raf")
                })
                .unwrap_or(false);
            
        let orientation = crate::t_image::get_image_orientation(&path_clone);
        let thumb_bytes = if is_raw {
            crate::t_image::get_raw_thumbnail(&path_clone, orientation, size)
        } else {
            crate::t_image::get_image_thumbnail(&path_clone, orientation, size)
        };
        
        match thumb_bytes {
            Ok(Some(bytes)) => {
                let encoded = general_purpose::STANDARD.encode(&bytes);
                Ok(format!("data:image/jpeg;base64,{}", encoded))
            }
            Ok(None) => Err("No thumbnail found".to_string()),
            Err(e) => Err(e),
        }
    })
    .await
    .map_err(|e| format!("Task joined with error: {}", e))?
}

// --- Вспомогательные функции ---

/// Определить разрешение изображения/видео по пути
fn get_file_resolution(path: &Path, extension: Option<&str>) -> Option<Resolution> {
    let ext = extension?;
    let is_image =
        t_common::NORMAL_IMGS.contains(&ext) || t_common::FFMPEG_BACKED_IMGS.contains(&ext);

    if is_image {
        if let Ok(dim) = imagesize::size(path) {
            return Some(Resolution {
                width: dim.width as u32,
                height: dim.height as u32,
            });
        }
    }

    None
}