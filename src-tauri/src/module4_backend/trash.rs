/**
 * .ams_trash — система безопасного удаления и undo.
 *
 * Вместо прямого удаления файлы перемещаются во внутреннюю корзину .ams_trash.
 * Оригиналы сохраняются во временную папку для возврата по Ctrl+Z.
 *
 * Tauri-команды:
 * - move_to_trash(paths) — переместить файлы в корзину
 * - restore_from_trash(paths) — восстановить из корзины
 * - get_trash_contents() — список файлов в корзине
 * - empty_trash() — очистить корзину
 * - backup_originals(paths) — сохранить копии оригиналов перед batch-операцией
 * - restore_originals(paths) — восстановить оригиналы
 */

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use tauri::command;

/// Запись в корзине
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashEntry {
    pub original_path: String,
    pub trash_path: String,
    pub deleted_at: String,
    pub size: u64,
    #[serde(default)]
    pub is_dir: bool,
}

/// Путь к корзине для указанной директории
fn get_trash_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Cannot find home directory".to_string())?;
    let trash_dir = home.join(".ams_trash");
    Ok(trash_dir)
}

/// Инициализация корзины
fn ensure_trash_dir() -> Result<PathBuf, String> {
    let trash_dir = get_trash_dir()?;
    fs::create_dir_all(&trash_dir).map_err(|e| format!("Failed to create trash dir: {}", e))?;
    Ok(trash_dir)
}

/// Путь к временной папке для backup-оригиналов
fn get_temp_backup_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Cannot find home directory".to_string())?;
    let backup_dir = home.join(".ams_trash").join("_backups");
    Ok(backup_dir)
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_all(entry.path(), dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Переместить файлы в корзину
#[command]
pub fn move_to_trash(paths: Vec<String>) -> Result<Vec<TrashEntry>, String> {
    let trash_dir = ensure_trash_dir()?;
    let mut entries = Vec::new();
    let now = chrono::Utc::now().to_rfc3339();

    for path_str in &paths {
        let path = Path::new(path_str);

        if !path.exists() {
            continue;
        }

        // Создаём уникальное имя в корзине (используем timestamp + оригинальное имя)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let file_name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        let trash_name = format!("{}_{}", timestamp, file_name);
        let trash_path = trash_dir.join(&trash_name);

        let is_dir = path.is_dir();
        let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        // Перемещаем в корзину
        if let Err(e) = fs::rename(path, &trash_path) {
            if is_dir {
                if let Err(copy_err) = copy_dir_all(path, &trash_path) {
                    eprintln!("Failed to move directory to trash {}: {} / {}", path_str, e, copy_err);
                    continue;
                }
                let _ = fs::remove_dir_all(path);
            } else {
                if let Err(copy_err) = fs::copy(path, &trash_path) {
                    eprintln!("Failed to move file to trash {}: {} / {}", path_str, e, copy_err);
                    continue;
                }
                let _ = fs::remove_file(path);
            }
        }

        entries.push(TrashEntry {
            original_path: path_str.clone(),
            trash_path: trash_path.to_string_lossy().to_string(),
            deleted_at: now.clone(),
            size,
            is_dir,
        });

        // Сохраняем метаданные о перемещении
        save_trash_meta(&trash_path, &TrashEntry {
            original_path: path_str.clone(),
            trash_path: trash_path.to_string_lossy().to_string(),
            deleted_at: now.clone(),
            size,
            is_dir,
        });
    }

    Ok(entries)
}

/// Восстановить файлы из корзины
#[command]
pub fn restore_from_trash(trash_paths: Vec<String>) -> Result<Vec<String>, String> {
    let mut restored = Vec::new();

    for trash_path_str in &trash_paths {
        let trash_path = Path::new(trash_path_str);

        if !trash_path.exists() {
            continue;
        }

        // Читаем метаданные для получения оригинального пути
        let meta = load_trash_meta(trash_path);

        let original_path = if let Some(ref meta) = meta {
            PathBuf::from(&meta.original_path)
        } else {
            // Если метаданных нет, восстанавливаем в домашнюю директорию
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.join(trash_path.file_name().unwrap_or_default())
        };

        // Убедимся, что родительская директория существует
        if let Some(parent) = original_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        // Перемещаем обратно
        if let Err(e) = fs::rename(trash_path, &original_path) {
            if trash_path.is_dir() {
                if let Err(copy_err) = copy_dir_all(trash_path, &original_path) {
                    eprintln!("Failed to restore directory {}: {} / {}", trash_path_str, e, copy_err);
                    continue;
                }
                let _ = fs::remove_dir_all(trash_path);
            } else {
                if let Err(copy_err) = fs::copy(trash_path, &original_path) {
                    eprintln!("Failed to restore file {}: {} / {}", trash_path_str, e, copy_err);
                    continue;
                }
                let _ = fs::remove_file(trash_path);
            }
        }

        // Удаляем файл метаданных
        let meta_path = trash_path.with_extension("meta.json");
        fs::remove_file(meta_path).ok();

        restored.push(original_path.to_string_lossy().to_string());
    }

    Ok(restored)
}

/// Восстановить файлы из корзины в другую папку по выбору пользователя
#[command]
pub fn restore_from_trash_to(trash_paths: Vec<String>, target_dir: String) -> Result<Vec<String>, String> {
    let mut restored = Vec::new();
    let target_dir_path = Path::new(&target_dir);
    if !target_dir_path.exists() {
        fs::create_dir_all(target_dir_path).map_err(|e| format!("Failed to create target directory: {}", e))?;
    }

    for trash_path_str in &trash_paths {
        let trash_path = Path::new(trash_path_str);

        if !trash_path.exists() {
            continue;
        }

        // Читаем метаданные для получения оригинального имени
        let meta = load_trash_meta(trash_path);
        let original_name = if let Some(ref meta) = meta {
            Path::new(&meta.original_path).file_name().unwrap_or(trash_path.file_name().unwrap_or_default())
        } else {
            trash_path.file_name().unwrap_or_default()
        };

        // Разрешаем конфликт имен, чтобы не перезаписать файл
        let dest_path = get_unique_dest_path(target_dir_path, original_name);

        // Перемещаем в выбранную папку
        if let Err(e) = fs::rename(trash_path, &dest_path) {
            if trash_path.is_dir() {
                if let Err(copy_err) = copy_dir_all(trash_path, &dest_path) {
                    eprintln!("Failed to restore directory to {:?}: {} / {}", dest_path, e, copy_err);
                    continue;
                }
                let _ = fs::remove_dir_all(trash_path);
            } else {
                if let Err(copy_err) = fs::copy(trash_path, &dest_path) {
                    eprintln!("Failed to restore file to {:?}: {} / {}", dest_path, e, copy_err);
                    continue;
                }
                let _ = fs::remove_file(trash_path);
            }
        }

        // Удаляем файл метаданных
        let meta_path = trash_path.with_extension("meta.json");
        fs::remove_file(meta_path).ok();

        restored.push(dest_path.to_string_lossy().to_string());
    }

    Ok(restored)
}

/// Получить уникальное имя файла/папки для предотвращения перезаписи при восстановлении
fn get_unique_dest_path(parent: &Path, file_name: &std::ffi::OsStr) -> PathBuf {
    let mut dest = parent.join(file_name);
    if !dest.exists() {
        return dest;
    }

    let stem = Path::new(file_name).file_stem().unwrap_or_default().to_string_lossy().to_string();
    let ext = Path::new(file_name).extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();

    let mut counter = 1;
    while dest.exists() {
        let new_name = format!("{}_{}{}", stem, counter, ext);
        dest = parent.join(new_name);
        counter += 1;
    }
    dest
}

/// Получить список файлов в корзине
#[command]
pub fn get_trash_contents() -> Result<Vec<TrashEntry>, String> {
    let trash_dir = ensure_trash_dir()?;
    let mut entries = Vec::new();

    let read_dir = fs::read_dir(&trash_dir).map_err(|e| format!("Failed to read trash: {}", e))?;

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        // Пропускаем файлы метаданных
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            continue;
        }

        if let Some(mut meta) = load_trash_meta(&path) {
            if !meta.is_dir && path.is_dir() {
                meta.is_dir = true;
            }
            entries.push(meta);
        } else {
            // Если метаданных нет, создаём запись с минимальной информацией
            let metadata = fs::metadata(&path).ok();
            let is_dir = path.is_dir();
            entries.push(TrashEntry {
                original_path: String::new(),
                trash_path: path.to_string_lossy().to_string(),
                deleted_at: String::new(),
                size: metadata.map(|m| m.len()).unwrap_or(0),
                is_dir,
            });
        }
    }

    Ok(entries)
}

/// Очистить корзину
#[command]
pub fn empty_trash() -> Result<(), String> {
    let trash_dir = ensure_trash_dir()?;

    let read_dir = fs::read_dir(&trash_dir).map_err(|e| format!("Failed to read trash: {}", e))?;

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).ok();
        } else {
            fs::remove_file(&path).ok();
        }
    }

    Ok(())
}

/// Сохранить копии оригиналов перед batch-операцией (для undo)
#[command]
pub fn backup_originals(paths: Vec<String>) -> Result<String, String> {
    let backup_dir = get_temp_backup_dir()?;
    let session_id = format!("batch_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos());
    let session_dir = backup_dir.join(&session_id);

    fs::create_dir_all(&session_dir).map_err(|e| format!("Failed to create backup dir: {}", e))?;

    for path_str in &paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let file_name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        let backup_path = session_dir.join(&*file_name);

        fs::copy(path, &backup_path).ok();
    }

    Ok(session_id)
}

/// Восстановить оригиналы из backup
#[command]
pub fn restore_originals(session_id: String) -> Result<(), String> {
    let backup_dir = get_temp_backup_dir()?;
    let session_dir = backup_dir.join(&session_id);

    if !session_dir.exists() {
        return Err(format!("Backup session not found: {}", session_id));
    }

    let read_dir = fs::read_dir(&session_dir).map_err(|e| format!("Failed to read backup: {}", e))?;

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let backup_path = entry.path();
        let file_name = backup_path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();

        // Пытаемся найти оригинал по тому же имени в исходной локации
        // (упрощённо — сохраняем поверх)
        let _original_path = &backup_path; // временно

        // В реальности нужно хранить mapping оригинальных путей
        // Пока просто восстанавливаем в ту же папку backup
        let restore_path = session_dir.parent().unwrap_or(&backup_dir).join("restored");
        fs::create_dir_all(&restore_path).ok();
        let dest = restore_path.join(&*file_name);
        fs::copy(&backup_path, &dest).ok();
    }

    Ok(())
}

// --- Вспомогательные функции ---

/// Сохранить метаданные о перемещении в корзину
fn save_trash_meta(trash_path: &Path, entry: &TrashEntry) {
    let meta_path = trash_path.with_extension("meta.json");
    if let Ok(json) = serde_json::to_string(entry) {
        fs::write(meta_path, json).ok();
    }
}

/// Загрузить метаданные о файле в корзине
fn load_trash_meta(trash_path: &Path) -> Option<TrashEntry> {
    let meta_path = trash_path.with_extension("meta.json");
    if meta_path.exists() {
        if let Ok(data) = fs::read_to_string(&meta_path) {
            if let Ok(entry) = serde_json::from_str::<TrashEntry>(&data) {
                return Some(entry);
            }
        }
    }
    None
}