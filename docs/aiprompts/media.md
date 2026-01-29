# Media 媒体处理

提供图片、PDF、SVG 等媒体文件的处理功能。

## 模块结构

```
media/
├── image.rs  # 图片处理
├── mime.rs   # MIME 类型
├── pdf.rs    # PDF 处理
└── svg.rs    # SVG 处理
```

## 媒体类型检测

```rust
pub enum MediaType {
    Image,
    Pdf,
    Svg,
    Unknown,
}

pub fn detect_media_type(file_path: &Path) -> MediaType;
pub fn is_supported_media_file(file_path: &Path) -> bool;
```

## 图片处理

```rust
pub fn read_image_file_enhanced(path: &Path) -> ImageResult;
pub fn estimate_image_dimensions(path: &Path) -> (u32, u32);
```


## PDF 处理

```rust
pub struct PdfReadResult {
    pub text: String,
    pub pages: usize,
}
```

## 二进制文件黑名单

不应被读取的文件类型：
- 音频: mp3, wav, flac...
- 视频: mp4, avi, mov...
- 压缩: zip, rar, tar...
- 可执行: exe, dll, so...

```rust
pub fn is_blacklisted_file(file_path: &Path) -> bool;
```

## 使用场景

- 图片内容理解
- PDF 文档解析
- 文件类型过滤

## 源码位置

`crates/aster/src/media/`
