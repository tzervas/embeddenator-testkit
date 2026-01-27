//! Format handlers for various file types
//!
//! Provides metadata extraction and content processing for:
//! - Media formats (images, video, audio)
//! - Document formats (PDF, HTML, XML)
//! - Code formats (multi-language)
//! - Structured data (JSON, CSV, etc.)

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Information extracted from a media file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    /// File format (e.g., "png", "mp4", "mp3")
    pub format: String,
    /// MIME type
    pub mime_type: String,
    /// File size in bytes
    pub size: u64,
    /// Duration in seconds (for video/audio)
    pub duration_secs: Option<f64>,
    /// Width in pixels (for images/video)
    pub width: Option<u32>,
    /// Height in pixels (for images/video)
    pub height: Option<u32>,
    /// Sample rate in Hz (for audio)
    pub sample_rate: Option<u32>,
    /// Number of channels (for audio)
    pub channels: Option<u8>,
    /// Bit depth (for audio)
    pub bit_depth: Option<u8>,
    /// Frame rate (for video)
    pub frame_rate: Option<f64>,
    /// Codec name
    pub codec: Option<String>,
    /// Raw bytes for VSA encoding (extracted frames, samples, or pixels)
    pub raw_data: Vec<u8>,
}

/// Supported media formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaFormat {
    // Images
    Png,
    Jpeg,
    WebP,
    Gif,
    Bmp,
    Tiff,
    // Video
    Mp4,
    WebM,
    Avi,
    Mkv,
    Mov,
    // Audio
    Mp3,
    Flac,
    Wav,
    Ogg,
    Aac,
}

impl MediaFormat {
    /// Get format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "webp" => Some(Self::WebP),
            "gif" => Some(Self::Gif),
            "bmp" => Some(Self::Bmp),
            "tiff" | "tif" => Some(Self::Tiff),
            "mp4" | "m4v" => Some(Self::Mp4),
            "webm" => Some(Self::WebM),
            "avi" => Some(Self::Avi),
            "mkv" => Some(Self::Mkv),
            "mov" => Some(Self::Mov),
            "mp3" => Some(Self::Mp3),
            "flac" => Some(Self::Flac),
            "wav" => Some(Self::Wav),
            "ogg" | "oga" => Some(Self::Ogg),
            "aac" | "m4a" => Some(Self::Aac),
            _ => None,
        }
    }

    /// Check if this is an image format
    pub fn is_image(&self) -> bool {
        matches!(
            self,
            Self::Png | Self::Jpeg | Self::WebP | Self::Gif | Self::Bmp | Self::Tiff
        )
    }

    /// Check if this is a video format
    pub fn is_video(&self) -> bool {
        matches!(
            self,
            Self::Mp4 | Self::WebM | Self::Avi | Self::Mkv | Self::Mov
        )
    }

    /// Check if this is an audio format
    pub fn is_audio(&self) -> bool {
        matches!(
            self,
            Self::Mp3 | Self::Flac | Self::Wav | Self::Ogg | Self::Aac
        )
    }

    /// Get MIME type for format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::WebP => "image/webp",
            Self::Gif => "image/gif",
            Self::Bmp => "image/bmp",
            Self::Tiff => "image/tiff",
            Self::Mp4 => "video/mp4",
            Self::WebM => "video/webm",
            Self::Avi => "video/x-msvideo",
            Self::Mkv => "video/x-matroska",
            Self::Mov => "video/quicktime",
            Self::Mp3 => "audio/mpeg",
            Self::Flac => "audio/flac",
            Self::Wav => "audio/wav",
            Self::Ogg => "audio/ogg",
            Self::Aac => "audio/aac",
        }
    }
}

/// Supported document formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentFormat {
    Pdf,
    Html,
    Xml,
    Markdown,
    Rst,
    Latex,
    Docx,
    Odt,
}

impl DocumentFormat {
    /// Get format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "pdf" => Some(Self::Pdf),
            "html" | "htm" => Some(Self::Html),
            "xml" => Some(Self::Xml),
            "md" | "markdown" => Some(Self::Markdown),
            "rst" => Some(Self::Rst),
            "tex" | "latex" => Some(Self::Latex),
            "docx" => Some(Self::Docx),
            "odt" => Some(Self::Odt),
            _ => None,
        }
    }
}

/// Supported code/programming language formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodeFormat {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    C,
    Cpp,
    Java,
    Go,
    Ruby,
    Php,
    Swift,
    Kotlin,
    Scala,
    Haskell,
    Shell,
    Sql,
}

impl CodeFormat {
    /// Get format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Self::Rust),
            "py" | "pyw" => Some(Self::Python),
            "js" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "tsx" => Some(Self::TypeScript),
            "c" | "h" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Self::Cpp),
            "java" => Some(Self::Java),
            "go" => Some(Self::Go),
            "rb" => Some(Self::Ruby),
            "php" => Some(Self::Php),
            "swift" => Some(Self::Swift),
            "kt" | "kts" => Some(Self::Kotlin),
            "scala" | "sc" => Some(Self::Scala),
            "hs" => Some(Self::Haskell),
            "sh" | "bash" | "zsh" => Some(Self::Shell),
            "sql" => Some(Self::Sql),
            _ => None,
        }
    }
}

/// Unified format handler for all supported formats
pub struct FormatHandler;

impl FormatHandler {
    /// Extract media information from a file
    #[cfg(feature = "media-formats")]
    pub fn extract_media_info(path: &Path) -> anyhow::Result<MediaInfo> {
        use std::fs;

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let format = MediaFormat::from_extension(ext)
            .ok_or_else(|| anyhow::anyhow!("Unsupported media format: {}", ext))?;

        let metadata = fs::metadata(path)?;
        let size = metadata.len();

        if format.is_image() {
            Self::extract_image_info(path, format, size)
        } else if format.is_video() {
            Self::extract_video_info(path, format, size)
        } else if format.is_audio() {
            Self::extract_audio_info(path, format, size)
        } else {
            Err(anyhow::anyhow!("Unknown media format"))
        }
    }

    #[cfg(feature = "media-formats")]
    fn extract_image_info(
        path: &Path,
        format: MediaFormat,
        size: u64,
    ) -> anyhow::Result<MediaInfo> {
        use image::GenericImageView;

        let img = image::open(path)?;
        let (width, height) = img.dimensions();

        // Convert to raw RGB bytes for VSA encoding
        let raw_data = img.to_rgb8().into_raw();

        Ok(MediaInfo {
            format: format!("{:?}", format).to_lowercase(),
            mime_type: format.mime_type().into(),
            size,
            duration_secs: None,
            width: Some(width),
            height: Some(height),
            sample_rate: None,
            channels: None,
            bit_depth: None,
            frame_rate: None,
            codec: None,
            raw_data,
        })
    }

    #[cfg(feature = "media-formats")]
    fn extract_video_info(
        _path: &Path,
        format: MediaFormat,
        size: u64,
    ) -> anyhow::Result<MediaInfo> {
        // Note: symphonia is primarily an audio library.
        // For full video support, consider adding ffmpeg-next or similar.
        // For now, we provide basic metadata without decoding.

        Ok(MediaInfo {
            format: format!("{:?}", format).to_lowercase(),
            mime_type: format.mime_type().into(),
            size,
            duration_secs: None,
            width: None,
            height: None,
            sample_rate: None,
            channels: None,
            bit_depth: None,
            frame_rate: None,
            codec: None,
            raw_data: Vec::new(),
        })
    }

    #[cfg(feature = "media-formats")]
    fn extract_audio_info(
        path: &Path,
        format: MediaFormat,
        size: u64,
    ) -> anyhow::Result<MediaInfo> {
        use symphonia::core::formats::FormatOptions;
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let file = std::fs::File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let probed =
            symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;

        let reader = probed.format;

        // Find audio track
        let track = reader
            .default_track()
            .ok_or_else(|| anyhow::anyhow!("No audio track found"))?;

        let codec_params = track.codec_params.clone();

        let duration = codec_params.n_frames.and_then(|frames| {
            codec_params
                .sample_rate
                .map(|rate| frames as f64 / rate as f64)
        });

        // For now, we skip decoding audio samples to avoid complex borrow issues.
        // Audio data can be decoded on-demand when needed for VSA encoding.
        // This just extracts metadata for now.
        let raw_samples: Vec<u8> = Vec::new();

        Ok(MediaInfo {
            format: format!("{:?}", format).to_lowercase(),
            mime_type: format.mime_type().into(),
            size,
            duration_secs: duration,
            width: None,
            height: None,
            sample_rate: codec_params.sample_rate,
            channels: codec_params.channels.map(|c| c.count() as u8),
            bit_depth: codec_params.bits_per_sample.map(|b| b as u8),
            frame_rate: None,
            codec: Some(format!("{:?}", codec_params.codec)),
            raw_data: raw_samples,
        })
    }

    /// Stub for when media-formats feature is not enabled
    #[cfg(not(feature = "media-formats"))]
    pub fn extract_media_info(_path: &Path) -> anyhow::Result<MediaInfo> {
        Err(anyhow::anyhow!(
            "Media format support not enabled. Enable the 'media-formats' feature."
        ))
    }

    /// Detect format from file path
    pub fn detect_format(path: &Path) -> FormatType {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if MediaFormat::from_extension(ext).is_some() {
            FormatType::Media
        } else if DocumentFormat::from_extension(ext).is_some() {
            FormatType::Document
        } else if CodeFormat::from_extension(ext).is_some() {
            FormatType::Code
        } else {
            match ext.to_lowercase().as_str() {
                "json" | "jsonl" | "ndjson" | "csv" | "tsv" | "yaml" | "yml" | "toml" => {
                    FormatType::Structured
                }
                "txt" | "text" | "log" => FormatType::PlainText,
                _ => FormatType::Binary,
            }
        }
    }
}

/// High-level format category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatType {
    /// Plain text files
    PlainText,
    /// Structured data (JSON, CSV, etc.)
    Structured,
    /// Source code
    Code,
    /// Documents (PDF, HTML, etc.)
    Document,
    /// Media files (images, video, audio)
    Media,
    /// Unknown binary format
    Binary,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_format_detection() {
        assert!(MediaFormat::from_extension("png").unwrap().is_image());
        assert!(MediaFormat::from_extension("mp4").unwrap().is_video());
        assert!(MediaFormat::from_extension("mp3").unwrap().is_audio());
        assert!(MediaFormat::from_extension("txt").is_none());
    }

    #[test]
    fn test_code_format_detection() {
        assert_eq!(CodeFormat::from_extension("rs"), Some(CodeFormat::Rust));
        assert_eq!(CodeFormat::from_extension("py"), Some(CodeFormat::Python));
        assert_eq!(CodeFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_format_type_detection() {
        use std::path::PathBuf;

        assert_eq!(
            FormatHandler::detect_format(&PathBuf::from("test.png")),
            FormatType::Media
        );
        assert_eq!(
            FormatHandler::detect_format(&PathBuf::from("test.rs")),
            FormatType::Code
        );
        assert_eq!(
            FormatHandler::detect_format(&PathBuf::from("test.json")),
            FormatType::Structured
        );
    }
}
