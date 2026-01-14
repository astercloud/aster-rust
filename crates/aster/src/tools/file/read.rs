//! Read Tool Implementation
//!
//! This module implements the `ReadTool` for reading files with:
//! - Text file reading with line numbers
//! - Image reading with base64 encoding
//! - PDF reading (optional)
//! - Jupyter notebook reading
//! - File read history tracking
//!
//! Requirements: 4.1, 4.2, 4.3, 4.4, 4.5

use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use super::{compute_content_hash, FileReadRecord, SharedFileReadHistory};
use crate::tools::base::{PermissionCheckResult, Tool};
use crate::tools::context::{ToolContext, ToolOptions, ToolResult};
use crate::tools::error::ToolError;

/// Maximum file size for text files (10MB)
pub const MAX_TEXT_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum file size for images (50MB)
pub const MAX_IMAGE_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Supported image extensions
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "svg"];

/// Supported text extensions (non-exhaustive, used for hints)
const TEXT_EXTENSIONS: &[&str] = &[
    "txt",
    "md",
    "rs",
    "py",
    "js",
    "ts",
    "jsx",
    "tsx",
    "json",
    "yaml",
    "yml",
    "toml",
    "xml",
    "html",
    "css",
    "scss",
    "less",
    "sql",
    "sh",
    "bash",
    "zsh",
    "c",
    "cpp",
    "h",
    "hpp",
    "java",
    "go",
    "rb",
    "php",
    "swift",
    "kt",
    "scala",
    "r",
    "lua",
    "pl",
    "pm",
    "ex",
    "exs",
    "erl",
    "hrl",
    "hs",
    "ml",
    "mli",
    "fs",
    "fsx",
    "clj",
    "cljs",
    "lisp",
    "el",
    "vim",
    "conf",
    "ini",
    "cfg",
    "env",
    "gitignore",
    "dockerignore",
    "makefile",
    "cmake",
    "gradle",
];

/// Line range for partial file reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineRange {
    /// Start line (1-indexed, inclusive)
    pub start: usize,
    /// End line (1-indexed, inclusive, None means to end of file)
    pub end: Option<usize>,
}

impl LineRange {
    /// Create a new line range
    pub fn new(start: usize, end: Option<usize>) -> Self {
        Self { start, end }
    }

    /// Create a range from start to end of file
    pub fn from_start(start: usize) -> Self {
        Self { start, end: None }
    }

    /// Create a range for a specific number of lines from start
    pub fn lines(start: usize, count: usize) -> Self {
        Self {
            start,
            end: Some(start + count - 1),
        }
    }
}

/// Read Tool for reading files
///
/// Supports reading:
/// - Text files with line numbers
/// - Images as base64
/// - PDF files (optional)
/// - Jupyter notebooks
///
/// Requirements: 4.1, 4.2, 4.3, 4.4, 4.5
#[derive(Debug)]
pub struct ReadTool {
    /// Shared file read history
    read_history: SharedFileReadHistory,
    /// Whether PDF reading is enabled
    pdf_enabled: bool,
}

impl ReadTool {
    /// Create a new ReadTool with shared history
    pub fn new(read_history: SharedFileReadHistory) -> Self {
        Self {
            read_history,
            pdf_enabled: false,
        }
    }

    /// Enable PDF reading
    pub fn with_pdf_enabled(mut self, enabled: bool) -> Self {
        self.pdf_enabled = enabled;
        self
    }

    /// Get the shared read history
    pub fn read_history(&self) -> &SharedFileReadHistory {
        &self.read_history
    }
}

// =============================================================================
// Text File Reading (Requirements: 4.1)
// =============================================================================

impl ReadTool {
    /// Read a text file with line numbers
    ///
    /// Returns the file content with line numbers prefixed.
    /// Optionally reads only a specific line range.
    ///
    /// Requirements: 4.1
    pub async fn read_text(
        &self,
        path: &Path,
        range: Option<LineRange>,
        context: &ToolContext,
    ) -> Result<String, ToolError> {
        let full_path = self.resolve_path(path, context);

        // Check file exists
        if !full_path.exists() {
            return Err(ToolError::execution_failed(format!(
                "File not found: {}",
                full_path.display()
            )));
        }

        // Check file size
        let metadata = fs::metadata(&full_path)?;
        if metadata.len() > MAX_TEXT_FILE_SIZE {
            return Err(ToolError::execution_failed(format!(
                "File too large: {} bytes (max: {} bytes)",
                metadata.len(),
                MAX_TEXT_FILE_SIZE
            )));
        }

        // Read file content
        let content = fs::read(&full_path)?;
        let text = String::from_utf8_lossy(&content);

        // Record the read
        self.record_file_read(&full_path, &content, &metadata)?;

        // Format with line numbers
        let lines: Vec<&str> = text.lines().collect();
        let total_lines = lines.len();

        let (start, end) = match range {
            Some(r) => {
                let start = r.start.saturating_sub(1).min(total_lines);
                let end = r.end.map(|e| e.min(total_lines)).unwrap_or(total_lines);
                (start, end)
            }
            None => (0, total_lines),
        };

        // Calculate line number width for formatting
        let line_width = (end.max(1)).to_string().len();

        let formatted: Vec<String> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let line_num = start + i + 1;
                format!("{:>width$} | {}", line_num, line, width = line_width)
            })
            .collect();

        debug!(
            "Read text file: {} ({} lines, showing {}-{})",
            full_path.display(),
            total_lines,
            start + 1,
            end
        );

        Ok(formatted.join("\n"))
    }

    /// Record a file read in the history
    fn record_file_read(
        &self,
        path: &Path,
        content: &[u8],
        metadata: &fs::Metadata,
    ) -> Result<(), ToolError> {
        let hash = compute_content_hash(content);
        let mtime = metadata.modified().ok();
        let line_count = String::from_utf8_lossy(content).lines().count();

        let mut record = FileReadRecord::new(path.to_path_buf(), hash, metadata.len())
            .with_line_count(line_count);

        if let Some(mt) = mtime {
            record = record.with_mtime(mt);
        }

        self.read_history.write().unwrap().record_read(record);
        Ok(())
    }

    /// Resolve a path relative to the working directory
    fn resolve_path(&self, path: &Path, context: &ToolContext) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            context.working_directory.join(path)
        }
    }
}

// =============================================================================
// Image Reading (Requirements: 4.2)
// =============================================================================

impl ReadTool {
    /// Read an image file and return base64 encoded content
    ///
    /// Requirements: 4.2
    pub async fn read_image(
        &self,
        path: &Path,
        context: &ToolContext,
    ) -> Result<String, ToolError> {
        let full_path = self.resolve_path(path, context);

        // Check file exists
        if !full_path.exists() {
            return Err(ToolError::execution_failed(format!(
                "Image not found: {}",
                full_path.display()
            )));
        }

        // Check file size
        let metadata = fs::metadata(&full_path)?;
        if metadata.len() > MAX_IMAGE_FILE_SIZE {
            return Err(ToolError::execution_failed(format!(
                "Image too large: {} bytes (max: {} bytes)",
                metadata.len(),
                MAX_IMAGE_FILE_SIZE
            )));
        }

        // Determine MIME type from extension
        let extension = full_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        let mime_type = match extension.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "bmp" => "image/bmp",
            "ico" => "image/x-icon",
            "svg" => "image/svg+xml",
            _ => "application/octet-stream",
        };

        // Read and encode
        let content = fs::read(&full_path)?;
        let base64_content = BASE64.encode(&content);

        // Record the read
        self.record_file_read(&full_path, &content, &metadata)?;

        debug!(
            "Read image file: {} ({} bytes, {})",
            full_path.display(),
            content.len(),
            mime_type
        );

        // Return as data URL
        Ok(format!("data:{};base64,{}", mime_type, base64_content))
    }

    /// Check if a file is an image based on extension
    pub fn is_image_file(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| IMAGE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
            .unwrap_or(false)
    }
}

// =============================================================================
// PDF Reading (Requirements: 4.3)
// =============================================================================

impl ReadTool {
    /// Read a PDF file (optional feature)
    ///
    /// Note: PDF reading requires external dependencies and is disabled by default.
    /// When enabled, extracts text content from PDF files.
    ///
    /// Requirements: 4.3
    pub async fn read_pdf(&self, path: &Path, context: &ToolContext) -> Result<String, ToolError> {
        if !self.pdf_enabled {
            return Err(ToolError::execution_failed(
                "PDF reading is not enabled. Enable it with ReadTool::with_pdf_enabled(true)",
            ));
        }

        let full_path = self.resolve_path(path, context);

        // Check file exists
        if !full_path.exists() {
            return Err(ToolError::execution_failed(format!(
                "PDF not found: {}",
                full_path.display()
            )));
        }

        // Read file content for history tracking
        let content = fs::read(&full_path)?;
        let metadata = fs::metadata(&full_path)?;
        self.record_file_read(&full_path, &content, &metadata)?;

        // PDF text extraction would go here
        // For now, return a placeholder indicating PDF support is limited
        warn!("PDF text extraction is not fully implemented");
        Ok(format!(
            "[PDF file: {} ({} bytes)]\n\
             PDF text extraction requires additional dependencies.\n\
             Consider using an external tool to convert PDF to text.",
            full_path.display(),
            content.len()
        ))
    }

    /// Check if a file is a PDF
    pub fn is_pdf_file(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase() == "pdf")
            .unwrap_or(false)
    }
}

// =============================================================================
// Jupyter Notebook Reading (Requirements: 4.4)
// =============================================================================

impl ReadTool {
    /// Read a Jupyter notebook file
    ///
    /// Extracts and formats code cells and markdown cells from the notebook.
    ///
    /// Requirements: 4.4
    pub async fn read_notebook(
        &self,
        path: &Path,
        context: &ToolContext,
    ) -> Result<String, ToolError> {
        let full_path = self.resolve_path(path, context);

        // Check file exists
        if !full_path.exists() {
            return Err(ToolError::execution_failed(format!(
                "Notebook not found: {}",
                full_path.display()
            )));
        }

        // Read and parse JSON
        let content = fs::read(&full_path)?;
        let metadata = fs::metadata(&full_path)?;

        let notebook: serde_json::Value = serde_json::from_slice(&content).map_err(|e| {
            ToolError::execution_failed(format!("Failed to parse notebook JSON: {}", e))
        })?;

        // Record the read
        self.record_file_read(&full_path, &content, &metadata)?;

        // Extract cells
        let cells = notebook
            .get("cells")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ToolError::execution_failed("Invalid notebook format: missing cells"))?;

        let mut output = Vec::new();
        output.push(format!("# Notebook: {}\n", full_path.display()));

        for (i, cell) in cells.iter().enumerate() {
            let cell_type = cell
                .get("cell_type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");

            let source = cell
                .get("source")
                .map(|s| self.extract_cell_source(s))
                .unwrap_or_default();

            match cell_type {
                "code" => {
                    output.push(format!("## Cell {} [code]\n```", i + 1));
                    output.push(source);
                    output.push("```\n".to_string());

                    // Include outputs if present
                    if let Some(outputs) = cell.get("outputs").and_then(|o| o.as_array()) {
                        if !outputs.is_empty() {
                            output.push("### Output:".to_string());
                            for out in outputs {
                                if let Some(text) = self.extract_output_text(out) {
                                    output.push(format!("```\n{}\n```", text));
                                }
                            }
                        }
                    }
                }
                "markdown" => {
                    output.push(format!("## Cell {} [markdown]\n", i + 1));
                    output.push(source);
                    output.push(String::new());
                }
                _ => {
                    output.push(format!("## Cell {} [{}]\n", i + 1, cell_type));
                    output.push(source);
                    output.push(String::new());
                }
            }
        }

        debug!(
            "Read notebook: {} ({} cells)",
            full_path.display(),
            cells.len()
        );

        Ok(output.join("\n"))
    }

    /// Extract source from a cell (handles both string and array formats)
    fn extract_cell_source(&self, source: &serde_json::Value) -> String {
        match source {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(""),
            _ => String::new(),
        }
    }

    /// Extract text from cell output
    fn extract_output_text(&self, output: &serde_json::Value) -> Option<String> {
        // Try "text" field first (stream output)
        if let Some(text) = output.get("text") {
            return Some(self.extract_cell_source(text));
        }

        // Try "data" -> "text/plain" (execute_result)
        if let Some(data) = output.get("data") {
            if let Some(text) = data.get("text/plain") {
                return Some(self.extract_cell_source(text));
            }
        }

        None
    }

    /// Check if a file is a Jupyter notebook
    pub fn is_notebook_file(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase() == "ipynb")
            .unwrap_or(false)
    }
}

// =============================================================================
// Tool Trait Implementation
// =============================================================================

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Read file contents. Supports text files (with line numbers), images (base64), \
         PDF files (optional), and Jupyter notebooks. Automatically detects file type \
         based on extension."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read (relative to working directory or absolute)"
                },
                "start_line": {
                    "type": "integer",
                    "description": "Start line number (1-indexed, for text files only)",
                    "minimum": 1
                },
                "end_line": {
                    "type": "integer",
                    "description": "End line number (1-indexed, inclusive, for text files only)",
                    "minimum": 1
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        // Check for cancellation
        if context.is_cancelled() {
            return Err(ToolError::Cancelled);
        }

        // Extract path parameter
        let path_str = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::invalid_params("Missing required parameter: path"))?;

        let path = Path::new(path_str);

        // Determine file type and read accordingly
        if Self::is_image_file(path) {
            let content = self.read_image(path, context).await?;
            return Ok(
                ToolResult::success(content).with_metadata("file_type", serde_json::json!("image"))
            );
        }

        if Self::is_pdf_file(path) {
            let content = self.read_pdf(path, context).await?;
            return Ok(
                ToolResult::success(content).with_metadata("file_type", serde_json::json!("pdf"))
            );
        }

        if Self::is_notebook_file(path) {
            let content = self.read_notebook(path, context).await?;
            return Ok(ToolResult::success(content)
                .with_metadata("file_type", serde_json::json!("notebook")));
        }

        // Default to text file
        let range = self.extract_line_range(&params);
        let content = self.read_text(path, range, context).await?;

        Ok(ToolResult::success(content).with_metadata("file_type", serde_json::json!("text")))
    }

    async fn check_permissions(
        &self,
        params: &serde_json::Value,
        context: &ToolContext,
    ) -> PermissionCheckResult {
        // Extract path for permission check
        let path_str = match params.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return PermissionCheckResult::deny("Missing path parameter"),
        };

        let path = Path::new(path_str);
        let full_path = self.resolve_path(path, context);

        // Check if path is within allowed directories
        // For now, allow all reads (permission manager handles restrictions)
        debug!("Permission check for read: {}", full_path.display());

        PermissionCheckResult::allow()
    }

    fn options(&self) -> ToolOptions {
        ToolOptions::new()
            .with_max_retries(1)
            .with_base_timeout(std::time::Duration::from_secs(30))
    }
}

impl ReadTool {
    /// Extract line range from parameters
    fn extract_line_range(&self, params: &serde_json::Value) -> Option<LineRange> {
        let start = params
            .get("start_line")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let end = params
            .get("end_line")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        match (start, end) {
            (Some(s), e) => Some(LineRange::new(s, e)),
            (None, Some(e)) => Some(LineRange::new(1, Some(e))),
            (None, None) => None,
        }
    }

    /// Check if a file is likely a text file based on extension
    pub fn is_text_file(path: &Path) -> bool {
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => {
                let ext_lower = ext.to_lowercase();
                // If it's a known text extension, return true
                // If it's a known non-text extension (image, pdf, notebook), return false
                // Otherwise, default to true (assume text)
                if TEXT_EXTENSIONS.contains(&ext_lower.as_str()) {
                    true
                } else if IMAGE_EXTENSIONS.contains(&ext_lower.as_str())
                    || ext_lower == "pdf"
                    || ext_lower == "ipynb"
                {
                    false
                } else {
                    true // Unknown extensions default to text
                }
            }
            None => true, // No extension defaults to text
        }
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_context(dir: &Path) -> ToolContext {
        ToolContext::new(dir.to_path_buf())
            .with_session_id("test-session")
            .with_user("test-user")
    }

    fn create_read_tool() -> ReadTool {
        ReadTool::new(super::super::create_shared_history())
    }

    #[test]
    fn test_line_range_new() {
        let range = LineRange::new(5, Some(10));
        assert_eq!(range.start, 5);
        assert_eq!(range.end, Some(10));
    }

    #[test]
    fn test_line_range_from_start() {
        let range = LineRange::from_start(5);
        assert_eq!(range.start, 5);
        assert_eq!(range.end, None);
    }

    #[test]
    fn test_line_range_lines() {
        let range = LineRange::lines(5, 10);
        assert_eq!(range.start, 5);
        assert_eq!(range.end, Some(14));
    }

    #[test]
    fn test_is_image_file() {
        assert!(ReadTool::is_image_file(Path::new("test.png")));
        assert!(ReadTool::is_image_file(Path::new("test.jpg")));
        assert!(ReadTool::is_image_file(Path::new("test.JPEG")));
        assert!(ReadTool::is_image_file(Path::new("test.gif")));
        assert!(!ReadTool::is_image_file(Path::new("test.txt")));
        assert!(!ReadTool::is_image_file(Path::new("test.rs")));
    }

    #[test]
    fn test_is_pdf_file() {
        assert!(ReadTool::is_pdf_file(Path::new("test.pdf")));
        assert!(ReadTool::is_pdf_file(Path::new("test.PDF")));
        assert!(!ReadTool::is_pdf_file(Path::new("test.txt")));
    }

    #[test]
    fn test_is_notebook_file() {
        assert!(ReadTool::is_notebook_file(Path::new("test.ipynb")));
        assert!(ReadTool::is_notebook_file(Path::new("test.IPYNB")));
        assert!(!ReadTool::is_notebook_file(Path::new("test.py")));
    }

    #[test]
    fn test_is_text_file() {
        assert!(ReadTool::is_text_file(Path::new("test.txt")));
        assert!(ReadTool::is_text_file(Path::new("test.rs")));
        assert!(ReadTool::is_text_file(Path::new("test.py")));
        assert!(ReadTool::is_text_file(Path::new("test.json")));
        // Unknown extensions default to text
        assert!(ReadTool::is_text_file(Path::new("test.unknown")));
    }

    #[tokio::test]
    async fn test_read_text_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create test file
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "Line 1").unwrap();
        writeln!(file, "Line 2").unwrap();
        writeln!(file, "Line 3").unwrap();

        let tool = create_read_tool();
        let context = create_test_context(temp_dir.path());

        let result = tool.read_text(&file_path, None, &context).await.unwrap();

        assert!(result.contains("1 | Line 1"));
        assert!(result.contains("2 | Line 2"));
        assert!(result.contains("3 | Line 3"));

        // Check history was recorded
        assert!(tool.read_history.read().unwrap().has_read(&file_path));
    }

    #[tokio::test]
    async fn test_read_text_file_with_range() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create test file with 10 lines
        let mut file = fs::File::create(&file_path).unwrap();
        for i in 1..=10 {
            writeln!(file, "Line {}", i).unwrap();
        }

        let tool = create_read_tool();
        let context = create_test_context(temp_dir.path());

        let range = LineRange::new(3, Some(5));
        let result = tool
            .read_text(&file_path, Some(range), &context)
            .await
            .unwrap();

        assert!(result.contains("3 | Line 3"));
        assert!(result.contains("4 | Line 4"));
        assert!(result.contains("5 | Line 5"));
        assert!(!result.contains("Line 1"));
        assert!(!result.contains("Line 6"));
    }

    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");

        let tool = create_read_tool();
        let context = create_test_context(temp_dir.path());

        let result = tool.read_text(&file_path, None, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_image_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.png");

        // Create a minimal PNG file (1x1 transparent pixel)
        let png_data: Vec<u8> = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
            0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, // IDAT chunk
            0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D,
            0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, // IEND chunk
            0x42, 0x60, 0x82,
        ];
        fs::write(&file_path, &png_data).unwrap();

        let tool = create_read_tool();
        let context = create_test_context(temp_dir.path());

        let result = tool.read_image(&file_path, &context).await.unwrap();

        assert!(result.starts_with("data:image/png;base64,"));
        assert!(tool.read_history.read().unwrap().has_read(&file_path));
    }

    #[tokio::test]
    async fn test_read_notebook_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ipynb");

        // Create a minimal notebook
        let notebook = serde_json::json!({
            "cells": [
                {
                    "cell_type": "code",
                    "source": ["print('Hello')"],
                    "outputs": []
                },
                {
                    "cell_type": "markdown",
                    "source": ["# Title"]
                }
            ],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 2
        });
        fs::write(&file_path, serde_json::to_string(&notebook).unwrap()).unwrap();

        let tool = create_read_tool();
        let context = create_test_context(temp_dir.path());

        let result = tool.read_notebook(&file_path, &context).await.unwrap();

        assert!(result.contains("Cell 1 [code]"));
        assert!(result.contains("print('Hello')"));
        assert!(result.contains("Cell 2 [markdown]"));
        assert!(result.contains("# Title"));
    }

    #[tokio::test]
    async fn test_tool_execute_text() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        let tool = create_read_tool();
        let context = create_test_context(temp_dir.path());
        let params = serde_json::json!({
            "path": file_path.to_str().unwrap()
        });

        let result = tool.execute(params, &context).await.unwrap();

        assert!(result.is_success());
        assert!(result.output.unwrap().contains("Hello, World!"));
        assert_eq!(
            result.metadata.get("file_type"),
            Some(&serde_json::json!("text"))
        );
    }

    #[tokio::test]
    async fn test_tool_execute_with_line_range() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let mut file = fs::File::create(&file_path).unwrap();
        for i in 1..=10 {
            writeln!(file, "Line {}", i).unwrap();
        }

        let tool = create_read_tool();
        let context = create_test_context(temp_dir.path());
        let params = serde_json::json!({
            "path": file_path.to_str().unwrap(),
            "start_line": 2,
            "end_line": 4
        });

        let result = tool.execute(params, &context).await.unwrap();

        assert!(result.is_success());
        let output = result.output.unwrap();
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
        assert!(output.contains("Line 4"));
        assert!(!output.contains("Line 1"));
        assert!(!output.contains("Line 5"));
    }

    #[tokio::test]
    async fn test_tool_execute_missing_path() {
        let temp_dir = TempDir::new().unwrap();
        let tool = create_read_tool();
        let context = create_test_context(temp_dir.path());
        let params = serde_json::json!({});

        let result = tool.execute(params, &context).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidParams(_)));
    }

    #[test]
    fn test_tool_name() {
        let tool = create_read_tool();
        assert_eq!(tool.name(), "read");
    }

    #[test]
    fn test_tool_description() {
        let tool = create_read_tool();
        assert!(!tool.description().is_empty());
        assert!(tool.description().contains("Read"));
    }

    #[test]
    fn test_tool_input_schema() {
        let tool = create_read_tool();
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["path"].is_object());
        assert!(schema["properties"]["start_line"].is_object());
        assert!(schema["properties"]["end_line"].is_object());
    }

    #[tokio::test]
    async fn test_check_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let tool = create_read_tool();
        let context = create_test_context(temp_dir.path());
        let params = serde_json::json!({
            "path": "test.txt"
        });

        let result = tool.check_permissions(&params, &context).await;
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_check_permissions_missing_path() {
        let temp_dir = TempDir::new().unwrap();
        let tool = create_read_tool();
        let context = create_test_context(temp_dir.path());
        let params = serde_json::json!({});

        let result = tool.check_permissions(&params, &context).await;
        assert!(result.is_denied());
    }
}
