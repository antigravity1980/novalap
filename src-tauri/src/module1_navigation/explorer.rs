/**
 * Explorer Mode: чтение файловой системы «на лету».
 *
 * Tauri-команды:
 * - list_directory(path) — список содержимого директории
 * - get_drives() — список дисков (Windows) / mount points (Linux/macOS)
 * - get_file_info(path) — детальная информация о файле
 */

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::command;

use crate::t_common;

/// Тип записи в файловой системе
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

/// Информация о диске / mount point
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
        .map(|path| DriveInfo {
            name: path.clone(),
            path,
            total_space: 0,
            free_space: 0,
            is_removable: path.contains("media") || path.contains("Volumes") || path == "/mnt",
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
pub fn get_file_info(path: String) -> Result<FileEntry, String> {
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
    })
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