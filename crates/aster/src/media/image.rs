//! 图片处理模块
//!

use base64::{engine::general_purpose::STANDARD, Engine};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use super::mime::get_mime_type_sync;

/// 支持的图片格式
pub static SUPPORTED_IMAGE_FORMATS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["png", "jpg", "jpeg", "gif", "webp"]));

/// 最大图片 token 数
pub const MAX_IMAGE_TOKENS: u64 = 25000;

/// 图片压缩配置
pub struct ImageCompressionConfig {
    pub max_width: u32,
    pub max_height: u32,
    pub quality: u8,
}

pub const IMAGE_COMPRESSION_CONFIG: ImageCompressionConfig = ImageCompressionConfig {
    max_width: 400,
    max_height: 400,
    quality: 20,
};

/// 图片尺寸信息
#[derive(Debug, Clone, Default)]
pub struct ImageDimensions {
    pub original_width: Option<u32>,
    pub original_height: Option<u32>,
    pub display_width: Option<u32>,
    pub display_height: Option<u32>,
}

/// 图片处理结果
#[derive(Debug, Clone)]
pub struct ImageResult {
    pub base64: String,
    pub mime_type: String,
    pub original_size: u64,
    pub dimensions: Option<ImageDimensions>,
}

/// 检查是否为支持的图片格式
pub fn is_supported_image_format(ext: &str) -> bool {
    let normalized = ext.to_lowercase().replace('.', "");
    SUPPORTED_IMAGE_FORMATS.contains(normalized.as_str())
}

/// 估算图片的 token 消耗
pub fn estimate_image_tokens(base64: &str) -> u64 {
    (base64.len() as f64 * 0.125).ceil() as u64
}

/// 读取图片文件（同步版本，不压缩）
pub fn read_image_file_sync(file_path: &Path) -> Result<ImageResult, String> {
    let metadata =
        fs::metadata(file_path).map_err(|e| format!("Failed to read file metadata: {}", e))?;

    if metadata.len() == 0 {
        return Err(format!("Image file is empty: {}", file_path.display()));
    }

    let buffer = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    let mime_type = get_mime_type_sync(&buffer)
        .unwrap_or_else(|| Box::leak(format!("image/{}", ext).into_boxed_str()));

    let base64 = STANDARD.encode(&buffer);

    Ok(ImageResult {
        base64,
        mime_type: mime_type.to_string(),
        original_size: metadata.len(),
        dimensions: None,
    })
}

/// 验证图片文件
pub fn validate_image_file(file_path: &Path) -> Result<(), String> {
    if !file_path.exists() {
        return Err("File does not exist".to_string());
    }

    let metadata =
        fs::metadata(file_path).map_err(|e| format!("Failed to read metadata: {}", e))?;

    if metadata.len() == 0 {
        return Err("Image file is empty".to_string());
    }

    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    if !is_supported_image_format(ext) {
        return Err(format!(
            "Unsupported image format: {}. Supported: {:?}",
            ext,
            SUPPORTED_IMAGE_FORMATS.iter().collect::<Vec<_>>()
        ));
    }

    Ok(())
}
