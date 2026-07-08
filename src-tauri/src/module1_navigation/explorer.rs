/**
 * Explorer Mode: чтение файловой системы "на лету".
 *
 * Tauri-команды:
 * - list_directory(path) - список содержимого директории
 * - get_drives() - список дисков (Windows) / mount points (Linux/macOS)
 * - get_file_info(path) - детальная информация о файле
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
    pub ai_source: Option<String>,
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

/// Счётчики / AI-источник, возвращаемые при ленивой догрузке видимых строк.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EntryEnrichment {
    pub path: String,
    pub dir_count: Option<u32>,
    pub file_count: Option<u32>,
    pub ai_source: Option<String>,
    pub resolution: Option<Resolution>,
    pub duration: Option<i64>,
    pub has_audio: Option<bool>,
}

/// Список файлов и директорий по пути
#[command]
pub async fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    tokio::task::spawn_blocking(move || {
        let dir_path = Path::new(&path);

        if !dir_path.exists() {
            return Err(format!("Path does not exist: {}", path));
        }
        if !dir_path.is_dir() {
            return Err(format!("Path is not a directory: {}", path));
        }

        let mut entries: Vec<FileEntry> = Vec::new();
        let read_dir =
            fs::read_dir(dir_path).map_err(|e| format!("Failed to read directory: {}", e))?;

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

            let created = metadata.created().ok().and_then(|t| {
                let duration = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                    .map(|dt| dt.to_rfc3339())
            });

            let extension = entry_path
                .extension()
                .map(|ext| ext.to_string_lossy().to_lowercase());

            let resolution = None;

            let (dir_count, file_count) = (None, None);

            let ai_source = None;

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
                ai_source,
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
    })
    .await
    .map_err(|e| format!("Task joined with error: {}", e))?
}

/// Получить список дисков (Windows)
#[cfg(target_os = "windows")]
#[command]
pub fn get_drives() -> Result<Vec<DriveInfo>, String> {
    use win32_imports::{
        DRIVE_CDROM, DRIVE_FIXED, DRIVE_RAMDISK, DRIVE_REMOTE, DRIVE_REMOVABLE,
        GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDriveStringsW,
    };

    let buffer_len = unsafe { GetLogicalDriveStringsW(0, core::ptr::null_mut()) };
    if buffer_len == 0 {
        return Err("Failed to enumerate logical drives".to_string());
    }

    let mut buffer = vec![0u16; buffer_len as usize + 1];
    let written = unsafe { GetLogicalDriveStringsW(buffer.len() as u32, buffer.as_mut_ptr()) };
    if written == 0 {
        return Err("Failed to read logical drive strings".to_string());
    }

    let mut drives = Vec::new();
    let mut start = 0usize;

    while start < written as usize {
        let end = match buffer[start..].iter().position(|&c| c == 0) {
            Some(pos) => start + pos,
            None => break,
        };

        if end == start {
            break;
        }

        let drive_wide = &buffer[start..end];
        let path = String::from_utf16_lossy(drive_wide);
        let drive_type = unsafe { GetDriveTypeW(buffer[start..=end].as_ptr()) };

        if matches!(
            drive_type,
            DRIVE_FIXED | DRIVE_REMOVABLE | DRIVE_REMOTE | DRIVE_CDROM | DRIVE_RAMDISK
        ) {
            let mut free_space = 0u64;
            let mut total_space = 0u64;

            let _ = unsafe {
                GetDiskFreeSpaceExW(
                    buffer[start..=end].as_ptr(),
                    core::ptr::null_mut(),
                    &mut total_space,
                    &mut free_space,
                )
            };

            let name = path.trim_end_matches(['\\', '/']).to_string();
            drives.push(DriveInfo {
                name,
                path,
                total_space,
                free_space,
                is_removable: matches!(drive_type, DRIVE_REMOVABLE | DRIVE_CDROM),
            });
        }

        start = end + 1;
    }

    drives.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(drives)
}

/// Ленивая догрузка метаданных для видимых папок/файлов.
/// Возвращает `dir_count`/`file_count` для директорий и лёгкое
/// определение `ai_source` по пути для файлов (без парсинга метаданных).
#[command]
pub async fn enrich_entries(paths: Vec<String>) -> Result<Vec<EntryEnrichment>, String> {
    let mut join_set = tokio::task::JoinSet::new();
    for raw in paths {
        if raw.is_empty() {
            continue;
        }
        join_set.spawn(async move {
            let path = std::path::Path::new(&raw);
            if !path.exists() {
                return EntryEnrichment {
                    path: raw,
                    dir_count: None,
                    file_count: None,
                    ai_source: None,
                    resolution: None,
                    duration: None,
                    has_audio: None,
                };
            }
            let metadata = match std::fs::metadata(&raw) {
                Ok(m) => m,
                Err(_) => {
                    return EntryEnrichment {
                        path: raw,
                        dir_count: None,
                        file_count: None,
                        ai_source: None,
                        resolution: None,
                        duration: None,
                        has_audio: None,
                    };
                }
            };
            if metadata.is_dir() {
                let (dir_count, file_count) = count_children(path);
                EntryEnrichment {
                    path: raw,
                    dir_count: Some(dir_count),
                    file_count: Some(file_count),
                    ai_source: None,
                    resolution: None,
                    duration: None,
                    has_audio: None,
                }
            } else {
                let ai_source = quick_ai_source(&raw);
                let ext_str = path.extension().map(|e| e.to_string_lossy().to_lowercase());
                
                let is_video = if let Some(ext) = &ext_str {
                    crate::t_common::VIDEOS.contains(&ext.as_str())
                } else {
                    false
                };

                let (resolution, duration, has_audio) = if is_video {
                    if let Ok(video_meta) = crate::t_video::get_video_metadata_async(&raw).await {
                        (
                            Some(Resolution {
                                width: video_meta.width,
                                height: video_meta.height,
                            }),
                            Some(video_meta.duration as i64),
                            Some(video_meta.has_audio),
                        )
                    } else {
                        (None, None, None)
                    }
                } else {
                    (get_file_resolution(path, ext_str.as_deref()), None, None)
                };

                EntryEnrichment {
                    path: raw,
                    dir_count: None,
                    file_count: None,
                    ai_source,
                    resolution,
                    duration,
                    has_audio,
                }
            }
        });
    }
    let mut out = Vec::new();
    while let Some(res) = join_set.join_next().await {
        if let Ok(enrichment) = res {
            out.push(enrichment);
        }
    }
    Ok(out)
}

fn count_children(path: &Path) -> (u32, u32) {
    let read_dir = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return (0, 0),
    };
    let mut dc = 0u32;
    let mut fc = 0u32;
    for entry in read_dir.flatten() {
        if let Ok(meta) = entry.metadata() {
            if meta.is_dir() {
                dc += 1;
            } else if meta.is_file() {
                fc += 1;
            }
        }
    }
    (dc, fc)
}

fn quick_ai_source(path: &str) -> Option<String> {
    let lowered = path.to_lowercase();
    if lowered.contains("comfyui") {
        Some("ComfyUI".to_string())
    } else if lowered.contains("midjourney") {
        Some("Midjourney".to_string())
    } else if lowered.contains("stable_diffusion") || lowered.contains("stablediffusion") {
        Some("Stable Diffusion".to_string())
    } else if lowered.contains("dall-e") || lowered.contains("dalle") {
        Some("DALL-E".to_string())
    } else {
        None
    }
}

/// Получить список mount points (Unix)
#[cfg(target_family = "unix")]
#[command]
pub fn get_drives() -> Result<Vec<DriveInfo>, String> {
    let mut drives = vec![DriveInfo {
        name: "/".to_string(),
        path: "/".to_string(),
        total_space: 0,
        free_space: 0,
        is_removable: false,
    }];

    if let Some(home) = dirs::home_dir() {
        let path = home.to_string_lossy().to_string();
        drives.push(DriveInfo {
            name: "Home".to_string(),
            path,
            total_space: 0,
            free_space: 0,
            is_removable: false,
        });
    }

    collect_unix_mounts("/Volumes", 1, true, &mut drives);
    collect_unix_mounts("/media", 2, true, &mut drives);
    collect_unix_mounts("/run/media", 2, true, &mut drives);
    collect_unix_mounts("/mnt", 2, true, &mut drives);

    drives.sort_by(|a, b| {
        if a.path == "/" {
            std::cmp::Ordering::Less
        } else if b.path == "/" {
            std::cmp::Ordering::Greater
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });
    drives.dedup_by(|a, b| a.path == b.path);

    Ok(drives)
}

#[cfg(target_family = "unix")]
fn collect_unix_mounts(
    base_path: &str,
    depth: usize,
    is_removable: bool,
    drives: &mut Vec<DriveInfo>,
) {
    let base = Path::new(base_path);
    if !base.exists() || !base.is_dir() {
        return;
    }

    collect_unix_mount_entries(base, depth, is_removable, drives);
}

#[cfg(target_family = "unix")]
fn collect_unix_mount_entries(
    dir: &Path,
    depth: usize,
    is_removable: bool,
    drives: &mut Vec<DriveInfo>,
) {
    if depth == 0 {
        return;
    }

    let read_dir = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let path_str = path.to_string_lossy().to_string();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| path_str.clone());

        drives.push(DriveInfo {
            name,
            path: path_str,
            total_space: 0,
            free_space: 0,
            is_removable,
        });

        collect_unix_mount_entries(&path, depth - 1, is_removable, drives);
    }
}

#[cfg(not(any(target_os = "windows", target_family = "unix")))]
#[command]
pub fn get_drives() -> Result<Vec<DriveInfo>, String> {
    Ok(Vec::new())
}

#[cfg(target_os = "windows")]
mod win32_imports {
    #![allow(non_camel_case_types, non_snake_case, unused)]

    pub const DRIVE_REMOVABLE: u32 = 2;
    pub const DRIVE_FIXED: u32 = 3;
    pub const DRIVE_REMOTE: u32 = 4;
    pub const DRIVE_CDROM: u32 = 5;
    pub const DRIVE_RAMDISK: u32 = 6;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub fn GetLogicalDriveStringsW(nBufferLength: u32, lpBuffer: *mut u16) -> u32;
        pub fn GetDriveTypeW(lpRootPathName: *const u16) -> u32;
        pub fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }
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
            chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0).map(|dt| dt.to_rfc3339())
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

    let ai_source = if is_file {
        match crate::module4_backend::detect_ai_source(file_path.to_string_lossy().to_string()) {
            Ok(src) => {
                if src == "Unknown" {
                    None
                } else {
                    Some(src)
                }
            }
            Err(_) => None,
        }
    } else {
        None
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
        ai_source,
    })
}

/// Получить эскиз изображения в base64 (с дисковым кешем)
#[command]
pub async fn get_explorer_thumbnail(path: String, size: u32) -> Result<String, String> {
    use base64::{Engine, engine::general_purpose};
    let path_clone = path.clone();
    tokio::task::spawn_blocking(move || {
        // --- 1. Вычисляем ключ кеша на основе пути, mtime, размера файла и размера миниатюры ---
        let meta = fs::metadata(&path_clone).map_err(|e| e.to_string())?;
        let mtime: i64 = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let file_size: u64 = meta.len();

        let mut hasher = blake3::Hasher::new();
        hasher.update(b"explorer-thumb-v1");
        hasher.update(path_clone.as_bytes());
        hasher.update(&mtime.to_le_bytes());
        hasher.update(&file_size.to_le_bytes());
        hasher.update(&size.to_le_bytes());
        let hash = hasher.finalize().to_hex().to_string();

        // --- 2. Ищем файл в кеше ---
        let cache_root = crate::t_config::get_app_cache_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from(".cache"))
            .join("explorer_thumbs");
        let shard = &hash[0..2];
        let cache_file = cache_root.join(shard).join(format!("{}.jpg", hash));

        if cache_file.exists() {
            // Кеш найден - читаем готовые байты без декодирования изображения
            let bytes = fs::read(&cache_file).map_err(|e| e.to_string())?;
            let encoded = general_purpose::STANDARD.encode(&bytes);
            return Ok(format!("data:image/jpeg;base64,{}", encoded));
        }

        // --- 3. Кеш холодный - декодируем и масштабируем ---
        let p = Path::new(&path_clone);
        let ext = p.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let is_video = crate::t_common::VIDEOS.contains(&ext.as_str());

        let is_raw = crate::t_libraw::is_tiff_path(&path_clone)
            || matches!(
                ext.as_str(),
                "cr2" | "cr3" | "nef" | "arw" | "dng" | "orf" | "rw2" | "pef" | "raf"
            );

        let orientation = crate::t_image::get_image_orientation(&path_clone);
        let thumb_bytes = if is_video {
            crate::t_video::get_video_thumbnail_sync(&path_clone, size, None, None)
        } else if is_raw {
            crate::t_image::get_raw_thumbnail(&path_clone, orientation, size)
        } else {
            crate::t_image::get_image_thumbnail(&path_clone, orientation, size)
        };

        match thumb_bytes {
            Ok(Some(bytes)) => {
                // --- 4. Сохраняем в кеш асинхронно (без блокировки) ---
                let shard_dir = cache_root.join(shard);
                if fs::create_dir_all(&shard_dir).is_ok() {
                    let _ = fs::write(&cache_file, &bytes);
                }

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
        match imagesize::size(path) {
            Ok(dim) => {
                return Some(Resolution {
                    width: dim.width as u32,
                    height: dim.height as u32,
                });
            }
            Err(e) => {
                let log_msg = format!("imagesize failed for {:?}: {:?}\n", path, e);
                let _ = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(r"d:\NovaLAP\novalap\debug_log.txt")
                    .and_then(|mut f| {
                        use std::io::Write;
                        write!(f, "{}", log_msg)
                    });
            }
        }
    } else {
        let log_msg = format!("get_file_resolution skipped (not an image) for {:?}, ext={:?}\n", path, ext);
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(r"d:\NovaLAP\novalap\debug_log.txt")
            .and_then(|mut f| {
                use std::io::Write;
                write!(f, "{}", log_msg)
            });
    }

    None
}

static GLOBAL_WATCHER: std::sync::OnceLock<
    std::sync::Mutex<Option<(String, notify::RecommendedWatcher)>>,
> = std::sync::OnceLock::new();

fn get_global_watcher() -> &'static std::sync::Mutex<Option<(String, notify::RecommendedWatcher)>> {
    GLOBAL_WATCHER.get_or_init(|| std::sync::Mutex::new(None))
}

#[command]
pub fn watch_directory(app: tauri::AppHandle, path: String) -> Result<(), String> {
    use notify::Watcher;
    let watcher_mutex = get_global_watcher();
    let mut watcher_guard = watcher_mutex.lock().map_err(|e| e.to_string())?;

    // Если мы уже отслеживаем этот путь, ничего не делаем
    if let Some((current_path, _)) = &*watcher_guard {
        if current_path == &path {
            return Ok(());
        }
    }

    // Останавливаем старый вотчер
    *watcher_guard = None;

    let path_clone = path.clone();
    let app_clone = app.clone();

    // Создаем новый вотчер
    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                // Если произошло изменение (создание, удаление, изменение файлов)
                if event.kind.is_create() || event.kind.is_remove() || event.kind.is_modify() {
                    use tauri::Emitter;
                    let _ = app_clone.emit("directory-changed", path_clone.clone());
                }
            }
        })
        .map_err(|e| format!("Failed to create watcher: {}", e))?;

    // Начинаем отслеживание папки (нерекурсивно)
    watcher
        .watch(Path::new(&path), notify::RecursiveMode::NonRecursive)
        .map_err(|e| format!("Failed to watch directory: {}", e))?;

    // Сохраняем вотчер
    *watcher_guard = Some((path, watcher));

    Ok(())
}

