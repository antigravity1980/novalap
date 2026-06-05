/**
 * Cross-disk file operations.
 *
 * Tauri-команды:
 * - cross_copy(src, dest) — копирование между дисками
 * - cross_move(src, dest) — перемещение между дисками
 * - open_in_explorer(path) — открыть в системном проводнике
 * - create_folder(path) — создать папку
 * - delete_file_or_folder(path) — удалить файл/папку
 */

use std::fs;
use std::io;
use std::path::Path;
use tauri::command;

/// Копирование файла или папки между дисками
#[command]
pub fn cross_copy(src: String, dest: String) -> Result<(), String> {
    let src_path = Path::new(&src);
    let dest_path = Path::new(&dest);

    if !src_path.exists() {
        return Err(format!("Source does not exist: {}", src));
    }

    // Убедимся, что родительская папка назначения существует
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create destination parent: {}", e))?;
    }

    if src_path.is_dir() {
        copy_directory_recursive(src_path, dest_path)
            .map_err(|e| format!("Failed to copy directory: {}", e))
    } else {
        fs::copy(src_path, dest_path)
            .map_err(|e| format!("Failed to copy file: {}", e))?;
        Ok(())
    }
}

/// Перемещение файла или папки между дисками
#[command]
pub fn cross_move(src: String, dest: String) -> Result<(), String> {
    let src_path = Path::new(&src);
    let dest_path = Path::new(&dest);

    if !src_path.exists() {
        return Err(format!("Source does not exist: {}", src));
    }

    // Убедимся, что родительская папка назначения существует
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create destination parent: {}", e))?;
    }

    // Пробуем сначала rename (если на одном диске — мгновенно)
    if fs::rename(src_path, dest_path).is_ok() {
        return Ok(());
    }

    // Если rename не сработал (cross-disk) — копируем + удаляем
    if src_path.is_dir() {
        copy_directory_recursive(src_path, dest_path)?;
        fs::remove_dir_all(src_path).map_err(|e| format!("Failed to remove source directory after copy: {}", e))?;
    } else {
        fs::copy(src_path, dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;
        fs::remove_file(src_path).map_err(|e| format!("Failed to remove source file after copy: {}", e))?;
    }

    Ok(())
}

/// Создать папку
#[command]
pub fn create_folder(path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(|e| format!("Failed to create folder: {}", e))
}

/// Удалить файл или папку (в корзину системы)
#[command]
pub fn delete_file_system(path: String) -> Result<(), String> {
    let path_obj = Path::new(&path);

    if !path_obj.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    // Используем крейт trash для перемещения в корзину ОС
    trash::delete(path_obj).map_err(|e| format!("Failed to move to trash: {}", e))
}

/// Открыть в системном проводнике
#[command]
pub fn open_in_explorer(path: String) -> Result<(), String> {
    let path_obj = Path::new(&path);

    if !path_obj.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    opener::open(path_obj).map_err(|e| format!("Failed to open in explorer: {}", e))
}

// --- Вспомогательные функции ---

/// Рекурсивное копирование директории с прогрессом через буфер
fn copy_directory_recursive(src: &Path, dest: &Path) -> io::Result<()> {
    if !dest.exists() {
        fs::create_dir_all(dest)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let file_name = entry.file_name();
        let src_path = entry.path();
        let dest_path = dest.join(&file_name);

        if entry_type.is_dir() {
            copy_directory_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}