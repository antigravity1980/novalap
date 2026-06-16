#![allow(dead_code)]

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
    for path_str in &paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        #[cfg(target_os = "macos")]
        {
            use trash::macos::{DeleteMethod, TrashContextExtMacos};
            let mut context = trash::TrashContext::default();
            context.set_delete_method(DeleteMethod::NsFileManager);
            context.delete(path).map_err(|e| format!("Failed to move to trash {}: {}", path_str, e))?;
        }
        #[cfg(not(target_os = "macos"))]
        {
            trash::delete(path).map_err(|e| format!("Failed to move to trash {}: {}", path_str, e))?;
        }
    }
    Ok(Vec::new())
}

/// Восстановить файлы из корзины (заглушка)
#[command]
pub fn restore_from_trash(_trash_paths: Vec<String>) -> Result<Vec<String>, String> {
    Ok(Vec::new())
}

/// Восстановить файлы из корзины в другую папку по выбору пользователя (заглушка)
#[command]
pub fn restore_from_trash_to(_trash_paths: Vec<String>, _target_dir: String) -> Result<Vec<String>, String> {
    Ok(Vec::new())
}

/// Получить список файлов в корзине (заглушка)
#[command]
pub fn get_trash_contents() -> Result<Vec<TrashEntry>, String> {
    Ok(Vec::new())
}

/// Очистить корзину (заглушка)
#[command]
pub fn empty_trash() -> Result<(), String> {
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