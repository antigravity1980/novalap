/// Cross‑platform drag‑drop image URL extraction and system clipboard file reading/writing.

#[cfg(target_os = "macos")]
mod imp {
    use std::ffi::{c_char, CStr, CString};

    unsafe extern "C" {
        fn lap_get_drag_image_url() -> *const c_char;
        fn lap_read_clipboard_files() -> *const c_char;
        fn lap_write_clipboard_files(paths_str: *const c_char);
        fn lap_free_string(ptr: *const c_char);
    }

    pub fn get_drag_image_url() -> Option<String> {
        let ptr = unsafe { lap_get_drag_image_url() };
        if ptr.is_null() {
            return None;
        }
        let url = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        unsafe { lap_free_string(ptr) };
        Some(url)
    }

    pub fn read_clipboard_files() -> Result<Vec<String>, String> {
        let ptr = unsafe { lap_read_clipboard_files() };
        if ptr.is_null() {
            return Ok(Vec::new());
        }
        let s = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        unsafe { lap_free_string(ptr) };
        let paths = s.lines().map(|line| line.to_string()).filter(|line| !line.is_empty()).collect();
        Ok(paths)
    }

    pub fn write_clipboard_files(paths: Vec<String>) -> Result<(), String> {
        let paths_str = paths.join("\n");
        let c_str = CString::new(paths_str).map_err(|e| e.to_string())?;
        unsafe { lap_write_clipboard_files(c_str.as_ptr()) };
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod imp {
    pub fn get_drag_image_url() -> Option<String> {
        let mut clipboard = arboard::Clipboard::new().ok()?;
        let text = clipboard.get_text().ok()?;
        let text = text.trim();
        if text.starts_with("http://") || text.starts_with("https://") {
            Some(text.to_string())
        } else {
            None
        }
    }

    pub fn read_clipboard_files() -> Result<Vec<String>, String> {
        use clipboard_win::{formats, Clipboard, Getter};
        let _clip = Clipboard::new().map_err(|e| format!("Failed to open clipboard: {:?}", e))?;
        let mut files = Vec::new();
        let _ = formats::FileList.read_clipboard(&mut files);
        Ok(files)
    }

    pub fn write_clipboard_files(paths: Vec<String>) -> Result<(), String> {
        use clipboard_win::{formats, Clipboard, Setter};
        let _clip = Clipboard::new().map_err(|e| format!("Failed to open clipboard: {:?}", e))?;
        formats::FileList.write_clipboard(&paths).map_err(|e| format!("Failed to write clipboard files: {:?}", e))?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use std::io::Write;
    use std::process::{Command, Stdio};

    pub fn get_drag_image_url() -> Option<String> {
        let mut clipboard = arboard::Clipboard::new().ok()?;
        let text = clipboard.get_text().ok()?;
        let text = text.trim();
        if text.starts_with("http://") || text.starts_with("https://") {
            Some(text.to_string())
        } else {
            None
        }
    }

    pub fn read_clipboard_files() -> Result<Vec<String>, String> {
        // Try wl-paste first (Wayland)
        if let Ok(output) = Command::new("wl-paste").args(["-t", "text/uri-list"]).output() {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                let paths = parse_uri_list(&s);
                if !paths.is_empty() {
                    return Ok(paths);
                }
            }
        }
        // Try xclip (X11)
        if let Ok(output) = Command::new("xclip").args(["-selection", "clipboard", "-o", "-t", "text/uri-list"]).output() {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                let paths = parse_uri_list(&s);
                if !paths.is_empty() {
                    return Ok(paths);
                }
            }
        }
        Ok(Vec::new())
    }

    pub fn write_clipboard_files(paths: Vec<String>) -> Result<(), String> {
        let uris: Vec<String> = paths.iter().map(|p| {
            let mut path = p.clone();
            if !path.starts_with('/') {
                path = format!("/{}", path);
            }
            format!("file://{}", path)
        }).collect();
        let data = uris.join("\n");

        // Try wl-copy (Wayland)
        if let Ok(mut child) = Command::new("wl-copy")
            .args(["-t", "text/uri-list"])
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(data.as_bytes());
            }
            let _ = child.wait();
            return Ok(());
        }

        // Try xclip (X11)
        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard", "-t", "text/uri-list"])
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(data.as_bytes());
            }
            let _ = child.wait();
            return Ok(());
        }

        Ok(())
    }

    fn parse_uri_list(uri_list: &str) -> Vec<String> {
        uri_list
            .lines()
            .map(|line| line.trim())
            .filter(|line| line.starts_with("file://"))
            .map(|line| {
                let path = line.trim_start_matches("file://");
                url_decode(path)
            })
            .collect()
    }

    fn url_decode(s: &str) -> String {
        let mut res = String::new();
        let mut chars = s.chars();
        while let Some(ch) = chars.next() {
            if ch == '%' {
                let mut hex = String::new();
                if let Some(c1) = chars.next() {
                    hex.push(c1);
                }
                if let Some(c2) = chars.next() {
                    hex.push(c2);
                }
                if let Ok(val) = u8::from_str_radix(&hex, 16) {
                    res.push(val as char);
                } else {
                    res.push('%');
                    res.push_str(&hex);
                }
            } else {
                res.push(ch);
            }
        }
        res
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod imp {
    pub fn get_drag_image_url() -> Option<String> {
        None
    }
    pub fn read_clipboard_files() -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
    pub fn write_clipboard_files(_paths: Vec<String>) -> Result<(), String> {
        Ok(())
    }
}

pub fn get_drag_image_url() -> Option<String> {
    imp::get_drag_image_url()
}

pub fn read_clipboard_files() -> Result<Vec<String>, String> {
    imp::read_clipboard_files()
}

pub fn write_clipboard_files(paths: Vec<String>) -> Result<(), String> {
    imp::write_clipboard_files(paths)
}

#[tauri::command]
pub fn read_from_system_clipboard() -> Result<Vec<String>, String> {
    read_clipboard_files()
}

#[tauri::command]
pub fn write_to_system_clipboard(paths: Vec<String>) -> Result<(), String> {
    write_clipboard_files(paths)
}
