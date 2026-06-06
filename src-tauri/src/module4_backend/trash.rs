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

        let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        // Перемещаем в корзину
        if let Err(e) = fs::rename(path, &trash_path) {
            // Если rename не работает (cross-disk), копируем и удаляем
            if let Err(copy_err) = fs::copy(path, &trash_path) {
                eprintln!("Failed to move to trash {}: {} / {}", path_str, e, copy_err);
                continue;
            }
            let _ = fs::remove_file(path);
        }

        entries.push(TrashEntry {
            original_path: path_str.clone(),
            trash_path: trash_path.to_string_lossy().to_string(),
            deleted_at: now.clone(),
            size,
        });

        // Сохраняем метаданные о перемещении
        save_trash_meta(&trash_path, &TrashEntry {
            original_path: path_str.clone(),
            trash_path: trash_path.to_string_lossy().to_string(),
            deleted_at: now.clone(),
            size,
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
            // Пробуем копировать + удалить
            if let Err(copy_err) = fs::copy(trash_path, &original_path) {
                eprintln!("Failed to restore {}: {} / {}", trash_path_str, e, copy_err);
                continue;
            }
            fs::remove_file(trash_path).ok();
        }

        // Удаляем файл метаданных
        let meta_path = trash_path.with_extension("meta.json");
        fs::remove_file(meta_path).ok();

        restored.push(original_path.to_string_lossy().to_string());
    }

    Ok(restored)
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

        if let Some(meta) = load_trash_meta(&path) {
            entries.push(meta);
        } else {
            // Если метаданных нет, создаём запись с минимальной информацией
            let metadata = fs::metadata(&path).ok();
            entries.push(TrashEntry {
                original_path: String::new(),
                trash_path: path.to_string_lossy().to_string(),
                deleted_at: String::new(),
                size: metadata.map(|m| m.len()).unwrap_or(0),
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