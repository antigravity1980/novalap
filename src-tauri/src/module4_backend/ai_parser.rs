/**
 * AI Metadata Parser.
 *
 * Извлекает AI-метаданные из:
 * - PNG: tEXt/iTXt chunks (ComfyUI workflow)
 * - WebP: EXIF user-comments
 * - MP4: custom metadata boxes
 * - JPEG: EXIF user-comments
 *
 * Tauri-команды:
 * - parse_ai_metadata(path) -> AiMetadata
 * - detect_ai_source(path) -> String (определение источника генерации)
 */

use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::command;

/// AI-метаданные
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AiMetadata {
    pub positive_prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub seed: Option<i64>,
    pub steps: Option<i32>,
    pub cfg_scale: Option<f64>,
    pub model: Option<String>,
    pub loras: Vec<String>,
    pub workflow: Option<String>,   // ComfyUI workflow JSON
    pub source_engine: Option<String>, // ComfyUI, Midjourney, Nano Banana, etc.
    pub raw_metadata: Vec<RawMetadataEntry>,
}

/// Сырой блок метаданных
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawMetadataEntry {
    pub key: String,
    pub value: String,
}


/// Парсинг AI-метаданных из файла
#[command]
pub fn parse_ai_metadata(path: String) -> Result<AiMetadata, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() || !file_path.is_file() {
        return Err(format!("File does not exist: {}", path));
    }

    let ext = file_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "png" => parse_png_metadata(file_path),
        "webp" => parse_webp_metadata(file_path),
        "jpg" | "jpeg" => parse_jpeg_metadata(file_path),
        "avif" => parse_avif_metadata(file_path),
        "mp4" | "mkv" | "webm" | "mov" => parse_video_metadata(file_path),
        _ => Err(format!("Unsupported file format: {}", ext)),
    }
}

/// Определение источника генерации
#[command]
pub fn detect_ai_source(path: String) -> Result<String, String> {
    let metadata = parse_ai_metadata(path)?;
    Ok(detect_engine(&metadata))
}

// --- PNG парсер ---

fn parse_png_metadata(path: &Path) -> Result<AiMetadata, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let decoder = png::Decoder::new(file);
    let reader = decoder.read_info().map_err(|e| format!("PNG decode error: {}", e))?;
    let info = reader.info();

    let mut metadata = AiMetadata::default();
    let mut raw_entries = Vec::new();

    for chunk in &info.uncompressed_latin1_text {
        raw_entries.push(RawMetadataEntry {
            key: chunk.keyword.clone(),
            value: chunk.text.clone(),
        });
        parse_metadata_key(&mut metadata, &chunk.keyword, &chunk.text);
    }

    for chunk in &info.compressed_latin1_text {
        let text = chunk.get_text().unwrap_or_else(|_| String::new());
        raw_entries.push(RawMetadataEntry {
            key: chunk.keyword.clone(),
            value: text.clone(),
        });
        parse_metadata_key(&mut metadata, &chunk.keyword, &text);
    }

    for chunk in &info.utf8_text {
        let text = chunk.get_text().unwrap_or_else(|_| String::new());
        raw_entries.push(RawMetadataEntry {
            key: chunk.keyword.clone(),
            value: text.clone(),
        });
        parse_metadata_key(&mut metadata, &chunk.keyword, &text);
    }

    metadata.raw_metadata = raw_entries;
    metadata.source_engine = Some(detect_engine(&metadata));

    Ok(metadata)
}

// --- WebP парсер ---

fn parse_webp_metadata(path: &Path) -> Result<AiMetadata, String> {
    // Для WebP используем существующий EXIF парсер
    let mut file = std::fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut file_data = vec![0u8; 256 * 1024];
    let n = std::io::Read::read(&mut file, &mut file_data).map_err(|e| format!("Failed to read file: {}", e))?;
    file_data.truncate(n);
    let mut metadata = AiMetadata::default();
    let mut raw_entries = Vec::new();

    // Ищем EXIF блок в WebP
    if let Ok(exif_data) = extract_webp_exif(&file_data) {
        // Парсим EXIF как строку
        let exif_str = String::from_utf8_lossy(&exif_data);
        for line in exif_str.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                raw_entries.push(RawMetadataEntry {
                    key: key.clone(),
                    value: value.clone(),
                });
                parse_metadata_key(&mut metadata, &key, &value);
            }
        }
    }

    // Также ищем XMP данные
    if let Ok(xmp_data) = extract_webp_xmp(&file_data) {
        let xmp_str = String::from_utf8_lossy(&xmp_data);
        raw_entries.push(RawMetadataEntry {
            key: "XMP".to_string(),
            value: xmp_str.to_string(),
        });

        // Парсим XMP на предмет prompt
        if let Some(prompt) = extract_from_xml(&xmp_str, "prompt") {
            metadata.positive_prompt = Some(prompt);
        }
        if let Some(negative) = extract_from_xml(&xmp_str, "negative") {
            metadata.negative_prompt = Some(negative);
        }
        if let Some(workflow) = extract_from_xml(&xmp_str, "workflow") {
            metadata.workflow = Some(workflow);
        }
        if let Some(model) = extract_from_xml(&xmp_str, "model") {
            metadata.model = Some(model);
        }
    }

    metadata.raw_metadata = raw_entries;
    metadata.source_engine = Some(detect_engine(&metadata));

    Ok(metadata)
}

// --- JPEG парсер (binary EXIF reader, no external crate) ---

fn parse_jpeg_metadata(path: &Path) -> Result<AiMetadata, String> {
    let mut file = std::fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut data = vec![0u8; 256 * 1024];
    let n = std::io::Read::read(&mut file, &mut data).map_err(|e| format!("Failed to read file: {}", e))?;
    data.truncate(n);
    let mut metadata = AiMetadata::default();
    let mut raw_entries = Vec::new();

    // SOI marker check
    if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
        return Err("Not a valid JPEG file".to_string());
    }

    let mut pos = 2usize;
    while pos + 4 <= data.len() {
        if data[pos] != 0xFF {
            pos += 1;
            continue;
        }
        let marker = data[pos + 1];
        // Skip standalone 0xFF bytes
        if marker == 0x00 || marker == 0xFF {
            pos += 1;
            continue;
        }
        // SOS — end of metadata area
        if marker == 0xDA {
            break;
        }
        // Markers without length
        if matches!(marker, 0xD0..=0xD7) {
            pos += 2;
            continue;
        }
        if pos + 4 > data.len() {
            break;
        }
        let seg_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
        if seg_len < 2 || pos + 2 + seg_len > data.len() {
            break;
        }

        let seg_start = pos + 2;
        let seg_end = pos + 2 + seg_len;
        let seg_data = &data[seg_start..seg_end];

        // APP1 — EXIF data
        if marker == 0xE1 {
            if let Some(exif_text) = extract_jpeg_exif_text(seg_data) {
                raw_entries.push(RawMetadataEntry {
                    key: "EXIF".to_string(),
                    value: exif_text.clone(),
                });
                try_parse_json_metadata(&mut metadata, &exif_text);
                // Also try key:value parsing
                for line in exif_text.lines() {
                    if let Some((k, v)) = line.split_once(':') {
                        parse_metadata_key(&mut metadata, k.trim(), v.trim());
                    }
                }
            }
        }

        pos = seg_end;
    }

    metadata.raw_metadata = raw_entries;
    metadata.source_engine = Some(detect_engine(&metadata));
    Ok(metadata)
}

/// Extract readable text from JPEG EXIF block
fn extract_jpeg_exif_text(data: &[u8]) -> Option<String> {
    if data.len() < 6 {
        return None;
    }
    // Skip EXIF header
    let exif_data = &data[4..]; // skip "Exif\0\0" prefix

    // Simple scan for readable ASCII text sequences
    let mut result = String::new();
    let mut current = String::new();
    for &b in exif_data {
        if b >= 0x20 && b <= 0x7E {
            current.push(b as char);
        } else {
            if current.len() >= 4 {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&current);
            }
            current.clear();
        }
    }
    if current.len() >= 4 {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&current);
    }

    if result.is_empty() { None } else { Some(result) }
}

// --- Видео парсер ---

fn parse_video_metadata(path: &Path) -> Result<AiMetadata, String> {
    // Пока заглушка — для видео нужно использовать ffprobe
    // TODO: полноценный парсер MP4/MKV метаданных
    let mut metadata = AiMetadata::default();

    // Пробуем получить метаданные через системную команду ffprobe
    if let Ok(result) = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            path.to_str().unwrap_or(""),
        ])
        .output()
    {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            // Ищем в выводе поля с AI-метаданными
            if let Some(comment) = extract_ffprobe_tag(&stdout, "comment") {
                try_parse_json_metadata(&mut metadata, &comment);
            }
            if let Some(description) = extract_ffprobe_tag(&stdout, "description") {
                try_parse_json_metadata(&mut metadata, &description);
            }
            if let Some(workflow) = extract_ffprobe_tag(&stdout, "workflow") {
                metadata.workflow = Some(workflow);
            }
        }
    }

    metadata.source_engine = Some(detect_engine(&metadata));
    Ok(metadata)
}

// --- Вспомогательные функции ---

/// Парсинг ключа метаданных
fn parse_metadata_key(metadata: &mut AiMetadata, key: &str, value: &str) {
    match key.to_lowercase().as_str() {
        "prompt" | "positive" | "positive_prompt" | "pos" => {
            metadata.positive_prompt = Some(value.to_string());
        }
        "negative_prompt" | "negative" | "neg" => {
            metadata.negative_prompt = Some(value.to_string());
        }
        "seed" => {
            if let Ok(seed) = value.parse::<i64>() {
                metadata.seed = Some(seed);
            }
        }
        "steps" => {
            if let Ok(steps) = value.parse::<i32>() {
                metadata.steps = Some(steps);
            }
        }
        "cfg" | "cfg_scale" => {
            if let Ok(cfg) = value.parse::<f64>() {
                metadata.cfg_scale = Some(cfg);
            }
        }
        "model" | "checkpoint" | "ckpt_name" => {
            metadata.model = Some(value.to_string());
        }
        "lora" | "loras" => {
            metadata.loras.push(value.to_string());
        }
        "workflow" | "workflow_json" => {
            metadata.workflow = Some(value.to_string());
        }
        _ => {
            // Пробуем распарсить как JSON
            try_parse_json_metadata(metadata, value);
        }
    }
}

/// Попытка распарсить JSON-строку (ComfyUI workflow, etc.)
fn try_parse_json_metadata(metadata: &mut AiMetadata, json_str: &str) {
    // Убираем возможные кавычки в начале/конце
    let trimmed = json_str.trim().trim_matches('"');

    // Проверяем, похоже на JSON
    if !(trimmed.starts_with('{') || trimmed.starts_with('[')) {
        return;
    }

    // Пробуем распарсить как JSON
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(obj) = val.as_object() {
            for (k, v) in obj {
                let value_str = match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    _ => v.to_string(),
                };
                parse_metadata_key(metadata, k, &value_str);

                // Если это похоже на ComfyUI workflow, сохраняем целиком
                if k == "nodes" || k == "links" || k == "groups" {
                    metadata.workflow = Some(trimmed.to_string());
                }
            }
        }
    }
}

/// Определение источника генерации по метаданным
fn detect_engine(metadata: &AiMetadata) -> String {
    let all_text = format!(
        "{:?} {:?} {:?} {:?} {:?}",
        metadata.positive_prompt,
        metadata.negative_prompt,
        metadata.workflow,
        metadata.model,
        metadata.raw_metadata
    )
    .to_lowercase();

    if all_text.contains("comfyui") || metadata.workflow.is_some() {
        return "ComfyUI".to_string();
    }
    if all_text.contains("midjourney") {
        return "Midjourney".to_string();
    }
    if all_text.contains("nano banana") || all_text.contains("nanobanana") {
        return "Nano Banana".to_string();
    }
    if all_text.contains("gpt_image") || all_text.contains("gpt images") {
        return "GPT Images".to_string();
    }
    if all_text.contains("grok") {
        return "Grok Image".to_string();
    }
    if all_text.contains("stable diffusion") || all_text.contains("sd_") {
        return "Stable Diffusion".to_string();
    }
    if all_text.contains("dall-e") || all_text.contains("dalle") {
        return "DALL-E".to_string();
    }
    if all_text.contains("krita") || all_text.contains("krita_ai") {
        return "Krita AI".to_string();
    }
    if all_text.contains("forge") {
        return "Forge".to_string();
    }

    "Unknown".to_string()
}

// --- Формат-специфичные парсеры ---

/// Извлечение EXIF данных из WebP
fn extract_webp_exif(data: &[u8]) -> Result<Vec<u8>, String> {
    // WebP: RIFF + WEBP + chunks
    // EXIF chunk имеет тег "EXIF"
    find_webp_chunk(data, "EXIF")
}

/// Извлечение XMP данных из WebP
fn extract_webp_xmp(data: &[u8]) -> Result<Vec<u8>, String> {
    find_webp_chunk(data, "XMP ")
}

/// Поиск chunk в WebP файле
fn find_webp_chunk(data: &[u8], chunk_tag: &str) -> Result<Vec<u8>, String> {
    if data.len() < 12 {
        return Err("Invalid WebP file".to_string());
    }

    // RIFF header
    let riff = &data[0..4];
    if riff != b"RIFF" {
        return Err("Not a RIFF file".to_string());
    }

    let mut pos = 12; // после RIFF header (8) + WEBP (4)
    while pos + 8 <= data.len() {
        let tag = std::str::from_utf8(&data[pos..pos + 4]).unwrap_or("");
        let chunk_size = u32::from_le_bytes([
            data[pos + 4],
            data[pos + 5],
            data[pos + 6],
            data[pos + 7],
        ]) as usize;

        if tag == chunk_tag {
            let start = pos + 8;
            let end = std::cmp::min(start + chunk_size, data.len());
            return Ok(data[start..end].to_vec());
        }

        pos += 8 + chunk_size;
        if chunk_size % 2 == 1 {
            pos += 1; // padding byte
        }
    }

    Err(format!("Chunk {} not found", chunk_tag))
}

/// Извлечение тега из вывода ffprobe JSON
fn extract_ffprobe_tag(json_str: &str, tag_name: &str) -> Option<String> {
    // Простой поиск без полноценного JSON парсинга
    let search = format!("\"{}\" : \"", tag_name);
    if let Some(start) = json_str.find(&search) {
        let value_start = start + search.len();
        if let Some(end) = json_str[value_start..].find('\"') {
            return Some(json_str[value_start..value_start + end].to_string());
        }
    }
    None
}

/// Извлечение значения из XML/XMP
fn extract_from_xml(xml: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag);
    let close_tag = format!("</{}>", tag);

    if let Some(start) = xml.find(&open_tag) {
        let value_start = start + open_tag.len();
        if let Some(end) = xml[value_start..].find(&close_tag) {
            return Some(xml[value_start..value_start + end].to_string());
        }
    }

    None
}

/// AVIF metadata parser (XMP, JSON and EXIF scanners)
fn parse_avif_metadata(path: &Path) -> Result<AiMetadata, String> {
    let mut file = std::fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut file_data = vec![0u8; 512 * 1024];
    let n = std::io::Read::read(&mut file, &mut file_data).map_err(|e| format!("Failed to read file: {}", e))?;
    file_data.truncate(n);
    let mut metadata = AiMetadata::default();
    let mut raw_entries = Vec::new();

    // 1. Scan for XMP block in the binary data
    if let Some(xmp_str) = find_xmp_block(&file_data) {
        raw_entries.push(RawMetadataEntry {
            key: "XMP".to_string(),
            value: xmp_str.clone(),
        });
        if let Some(prompt) = extract_from_xml(&xmp_str, "prompt") {
            metadata.positive_prompt = Some(prompt);
        }
        if let Some(negative) = extract_from_xml(&xmp_str, "negative") {
            metadata.negative_prompt = Some(negative);
        }
        if let Some(workflow) = extract_from_xml(&xmp_str, "workflow") {
            metadata.workflow = Some(workflow);
        }
        if let Some(model) = extract_from_xml(&xmp_str, "model") {
            metadata.model = Some(model);
        }
    }

    // 2. Scan for embedded JSON (ComfyUI workflow)
    if let Some(json_str) = find_json_block(&file_data) {
        raw_entries.push(RawMetadataEntry {
            key: "JSON".to_string(),
            value: json_str.clone(),
        });
        try_parse_json_metadata(&mut metadata, &json_str);
    }

    // 3. Scan for EXIF-like ASCII text strings
    if let Some(exif_text) = extract_exif_text_fallback(&file_data) {
        raw_entries.push(RawMetadataEntry {
            key: "EXIF_Fallback".to_string(),
            value: exif_text.clone(),
        });
        for line in exif_text.lines() {
            if let Some((k, v)) = line.split_once(':') {
                parse_metadata_key(&mut metadata, k.trim(), v.trim());
            }
        }
    }

    metadata.raw_metadata = raw_entries;
    metadata.source_engine = Some(detect_engine(&metadata));
    Ok(metadata)
}

fn find_xmp_block(data: &[u8]) -> Option<String> {
    let start_tag = b"<x:xmpmeta";
    let end_tag = b"</x:xmpmeta>";
    
    let start_pos = data.windows(start_tag.len()).position(|w| w == start_tag)?;
    let end_pos = data.windows(end_tag.len()).position(|w| w == end_tag)? + end_tag.len();
    
    if end_pos > start_pos {
        Some(String::from_utf8_lossy(&data[start_pos..end_pos]).to_string())
    } else {
        None
    }
}

fn find_json_block(data: &[u8]) -> Option<String> {
    let pattern = b"\"prompt\":";
    let pos = data.windows(pattern.len()).position(|w| w == pattern)?;
    
    let mut start = pos;
    while start > 0 {
        start -= 1;
        if data[start] == b'{' {
            break;
        }
    }
    
    let mut end = start;
    let mut open_braces = 0;
    let mut in_string = false;
    let mut escaped = false;
    
    while end < data.len() {
        let b = data[end];
        if escaped {
            escaped = false;
        } else if b == b'\\' {
            escaped = true;
        } else if b == b'"' {
            in_string = !in_string;
        } else if !in_string {
            if b == b'{' {
                open_braces += 1;
            } else if b == b'}' {
                open_braces -= 1;
                if open_braces == 0 {
                    end += 1;
                    break;
                }
            }
        }
        end += 1;
    }
    
    if end > start && open_braces == 0 {
        Some(String::from_utf8_lossy(&data[start..end]).to_string())
    } else {
        None
    }
}

fn extract_exif_text_fallback(data: &[u8]) -> Option<String> {
    let mut result = String::new();
    let mut current = String::new();
    for &b in data {
        if b >= 0x20 && b <= 0x7E {
            current.push(b as char);
        } else {
            if current.len() >= 6 && current.contains(':') {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&current);
            }
            current.clear();
        }
    }
    if current.len() >= 6 && current.contains(':') {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&current);
    }
    
    if result.is_empty() { None } else { Some(result) }
}