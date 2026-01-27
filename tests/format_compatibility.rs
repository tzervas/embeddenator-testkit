//! Format compatibility tests for dataset module
//!
//! Tests that format detection and metadata extraction work correctly
//! for all supported file types.

#![cfg(feature = "realworld-datasets")]

use embeddenator_testkit::datasets::{
    CodeFormat, DocumentFormat, FormatHandler, FormatType, MediaFormat,
};
use std::path::PathBuf;

#[test]
fn test_image_format_detection() {
    let image_extensions = ["png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff", "tif"];

    for ext in image_extensions.iter() {
        let format = MediaFormat::from_extension(ext);
        assert!(format.is_some(), "Should detect image format for .{}", ext);
        assert!(
            format.unwrap().is_image(),
            "Format .{} should be classified as image",
            ext
        );
    }
}

#[test]
fn test_video_format_detection() {
    let video_extensions = ["mp4", "webm", "avi", "mkv", "mov"];

    for ext in video_extensions.iter() {
        let format = MediaFormat::from_extension(ext);
        assert!(format.is_some(), "Should detect video format for .{}", ext);
        assert!(
            format.unwrap().is_video(),
            "Format .{} should be classified as video",
            ext
        );
    }
}

#[test]
fn test_audio_format_detection() {
    let audio_extensions = ["mp3", "flac", "wav", "ogg", "aac", "m4a"];

    for ext in audio_extensions.iter() {
        let format = MediaFormat::from_extension(ext);
        assert!(format.is_some(), "Should detect audio format for .{}", ext);
        assert!(
            format.unwrap().is_audio(),
            "Format .{} should be classified as audio",
            ext
        );
    }
}

#[test]
fn test_document_format_detection() {
    let doc_tests = [
        ("pdf", DocumentFormat::Pdf),
        ("html", DocumentFormat::Html),
        ("htm", DocumentFormat::Html),
        ("xml", DocumentFormat::Xml),
        ("md", DocumentFormat::Markdown),
        ("markdown", DocumentFormat::Markdown),
        ("rst", DocumentFormat::Rst),
        ("tex", DocumentFormat::Latex),
        ("latex", DocumentFormat::Latex),
        ("docx", DocumentFormat::Docx),
        ("odt", DocumentFormat::Odt),
    ];

    for (ext, expected) in doc_tests.iter() {
        let format = DocumentFormat::from_extension(ext);
        assert!(
            format.is_some(),
            "Should detect document format for .{}",
            ext
        );
        assert_eq!(
            format.unwrap(),
            *expected,
            "Format .{} should be {:?}",
            ext,
            expected
        );
    }
}

#[test]
fn test_code_format_detection() {
    let code_tests = [
        ("rs", CodeFormat::Rust),
        ("py", CodeFormat::Python),
        ("pyw", CodeFormat::Python),
        ("js", CodeFormat::JavaScript),
        ("mjs", CodeFormat::JavaScript),
        ("ts", CodeFormat::TypeScript),
        ("tsx", CodeFormat::TypeScript),
        ("c", CodeFormat::C),
        ("h", CodeFormat::C),
        ("cpp", CodeFormat::Cpp),
        ("cc", CodeFormat::Cpp),
        ("hpp", CodeFormat::Cpp),
        ("java", CodeFormat::Java),
        ("go", CodeFormat::Go),
        ("rb", CodeFormat::Ruby),
        ("php", CodeFormat::Php),
        ("swift", CodeFormat::Swift),
        ("kt", CodeFormat::Kotlin),
        ("scala", CodeFormat::Scala),
        ("hs", CodeFormat::Haskell),
        ("sh", CodeFormat::Shell),
        ("bash", CodeFormat::Shell),
        ("sql", CodeFormat::Sql),
    ];

    for (ext, expected) in code_tests.iter() {
        let format = CodeFormat::from_extension(ext);
        assert!(format.is_some(), "Should detect code format for .{}", ext);
        assert_eq!(
            format.unwrap(),
            *expected,
            "Format .{} should be {:?}",
            ext,
            expected
        );
    }
}

#[test]
fn test_format_type_detection() {
    let type_tests = [
        ("test.png", FormatType::Media),
        ("test.mp4", FormatType::Media),
        ("test.mp3", FormatType::Media),
        ("test.rs", FormatType::Code),
        ("test.py", FormatType::Code),
        ("test.pdf", FormatType::Document),
        ("test.html", FormatType::Document),
        ("test.json", FormatType::Structured),
        ("test.csv", FormatType::Structured),
        ("test.yaml", FormatType::Structured),
        ("test.toml", FormatType::Structured),
        ("test.txt", FormatType::PlainText),
        ("test.log", FormatType::PlainText),
        ("test.unknown", FormatType::Binary),
    ];

    for (filename, expected) in type_tests.iter() {
        let path = PathBuf::from(filename);
        let detected = FormatHandler::detect_format(&path);
        assert_eq!(
            detected, *expected,
            "File {} should be detected as {:?}",
            filename, expected
        );
    }
}

#[test]
fn test_mime_types() {
    // Images
    assert_eq!(MediaFormat::Png.mime_type(), "image/png");
    assert_eq!(MediaFormat::Jpeg.mime_type(), "image/jpeg");
    assert_eq!(MediaFormat::WebP.mime_type(), "image/webp");
    assert_eq!(MediaFormat::Gif.mime_type(), "image/gif");

    // Video
    assert_eq!(MediaFormat::Mp4.mime_type(), "video/mp4");
    assert_eq!(MediaFormat::WebM.mime_type(), "video/webm");

    // Audio
    assert_eq!(MediaFormat::Mp3.mime_type(), "audio/mpeg");
    assert_eq!(MediaFormat::Flac.mime_type(), "audio/flac");
    assert_eq!(MediaFormat::Wav.mime_type(), "audio/wav");
}

#[test]
fn test_case_insensitivity() {
    // Extension matching should be case-insensitive
    assert!(MediaFormat::from_extension("PNG").is_some());
    assert!(MediaFormat::from_extension("JPG").is_some());
    assert!(MediaFormat::from_extension("Mp4").is_some());
    assert!(CodeFormat::from_extension("RS").is_some());
    assert!(CodeFormat::from_extension("Py").is_some());
    assert!(DocumentFormat::from_extension("PDF").is_some());
}

#[test]
fn test_unknown_formats_return_none() {
    assert!(MediaFormat::from_extension("xyz").is_none());
    assert!(MediaFormat::from_extension("").is_none());
    assert!(CodeFormat::from_extension("xyz").is_none());
    assert!(DocumentFormat::from_extension("xyz").is_none());
}
