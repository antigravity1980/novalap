/**
 * Tree View: навигационное дерево папок/дисков.
 *
 * Tauri-команды:
 * - get_tree(path) — получить поддерево для указанного пути
 * - get_parent_folders(path) — получить цепочку родительских папок
 * - expand_folder(path) — получить содержимое папки для поддерева
 */

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TreeFolder {
    pub name: String,
    pub path: String,
    pub has_subfolders: bool,
    pub children: Vec<TreeFolder>,
}

/// Получить дерево папок для указанного пути (только 1 уровень вложенности)
#[command]
pub fn expand_folder(path: String) -> Result<Vec<TreeFolder>, String> {
    let dir_path = Path::new(&path);

    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(format!("Invalid directory: {}", path));
    }

    let mut folders = Vec::new();
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

        if !metadata.is_dir() {
            continue;
        }

        // Skip hidden folders (starting with .)
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy().to_string();
        if name_str.starts_with('.') {
            continue;
        }

        let entry_path = entry.path();
        let path_str = entry_path.to_string_lossy().to_string();

        // Проверяем, есть ли внутри подпапки
        let has_subfolders = has_subdirectories(&entry_path);

        folders.push(TreeFolder {
            name: name_str,
            path: path_str,
            has_subfolders,
            children: Vec::new(),
        });
    }

    // Сортируем по имени
    folders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(folders)
}

/// Получить цепочку родительских папок (breadcrumb)
#[command]
pub fn get_parent_folders(path: String) -> Result<Vec<TreeFolder>, String> {
    let mut result = Vec::new();
    let mut current = Path::new(&path);

    loop {
        let name = current
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| current.to_string_lossy().to_string());

        let path_str = current.to_string_lossy().to_string();

        result.push(TreeFolder {
            name,
            path: path_str,
            has_subfolders: false,
            children: Vec::new(),
        });

        match current.parent() {
            Some(parent) => {
                if parent == current {
                    break; // reached root
                }
                current = parent;
            }
            None => break,
        }
    }

    result.reverse();
    Ok(result)
}

/// Получить корневые диски/папки
#[command]
pub fn get_root_folders() -> Result<Vec<TreeFolder>, String> {
    let mut roots = Vec::new();

    #[cfg(target_os = "windows")]
    {
        for letter in 'A'..='Z' {
            let path = format!("{}:\\", letter);
            let path_obj = Path::new(&path);
            if path_obj.exists() {
                let has_subfolders = has_subdirectories(path_obj);
                roots.push(TreeFolder {
                    name: format!("{}:", letter),
                    path,
                    has_subfolders,
                    children: Vec::new(),
                });
            }
        }
    }

    #[cfg(target_family = "unix")]
    {
        let root = TreeFolder {
            name: "/".to_string(),
            path: "/".to_string(),
            has_subfolders: true,
            children: Vec::new(),
        };
        roots.push(root);
    }

    Ok(roots)
}

// --- Вспомогательные функции ---

/// Проверить, есть ли в директории подпапки
fn has_subdirectories(path: &Path) -> bool {
    let read_dir = match fs::read_dir(path) {
        Ok(d) => d,
        Err(_) => return false,
    };

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if let Ok(metadata) = entry.metadata() {
            if metadata.is_dir() {
                let name = entry.file_name();
                if !name.to_string_lossy().starts_with('.') {
                    return true;
                }
            }
        }
    }

    false
}