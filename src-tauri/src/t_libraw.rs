use std::path::Path;

pub struct RawMeta {
    pub make: Option<String>,
    pub model: Option<String>,
    pub software: Option<String>,
    pub artist: Option<String>,
    pub description: Option<String>,
    pub timestamp: Option<i64>,
    pub iso_speed: Option<String>,
    pub shutter: Option<String>,
    pub aperture: Option<String>,
    pub focal_len: Option<String>,
    pub flash_used: Option<String>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
}

pub fn is_tiff_path(file_path: &str) -> bool {
    Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .map(|ext| ext == "tif" || ext == "tiff")
        .unwrap_or(false)
}

pub fn get_raw_dimensions(_file_path: &str) -> Result<(u32, u32), String> {
    Err("RAW format is not supported in LapAI".to_string())
}

pub fn get_raw_dimensions_with_flip(_file_path: &str) -> Result<(u32, u32, i32), String> {
    Err("RAW format is not supported in LapAI".to_string())
}

pub fn get_raw_meta(_file_path: &str) -> Result<RawMeta, String> {
    Err("RAW format is not supported in LapAI".to_string())
}

pub fn get_raw_preview_image(_file_path: &str) -> Result<Option<Vec<u8>>, String> {
    Ok(None)
}

pub fn get_raw_thumbnail(_file_path: &str, _thumbnail_size: u32) -> Result<Option<Vec<u8>>, String> {
    Ok(None)
}
