//! Real-world dataset management for comprehensive testing and benchmarking
//!
//! This module provides infrastructure for:
//! - Downloading and caching benchmark datasets
//! - Managing datasets across size tiers (Quick, Integration, Nightly, Release)
//! - Format-specific test data generation
//! - Industry-standard benchmark evaluation (MS MARCO, BEIR, etc.)
//!
//! # Dataset Tiers
//!
//! | Tier | Size | Use Case |
//! |------|------|----------|
//! | Quick | <100MB | CI/PR tests |
//! | Integration | 100MB-1GB | Daily integration tests |
//! | Nightly | 1-10GB | Nightly benchmarks |
//! | Release | 10-20GB | Release validation |
//!
//! # Supported Formats
//!
//! - **Text**: TXT, MD, RST, HTML
//! - **Structured**: JSON, JSONL, CSV, XML
//! - **Code**: Rust, Python, JavaScript, C/C++
//! - **Media**: Images (PNG, JPEG, WebP), Video (MP4, WebM), Audio (MP3, FLAC)
//! - **Documents**: PDF (metadata), Archives (ZIP, TAR)
//!
//! # Example
//!
//! ```rust,ignore
//! use embeddenator_testkit::datasets::{DatasetManager, DatasetTier};
//!
//! // Create manager
//! let mut manager = DatasetManager::new();
//!
//! // List quick datasets for CI
//! let quick_datasets = manager.list_by_tier(DatasetTier::Quick);
//!
//! // Download a specific dataset (async)
//! // let path = manager.download("wikitext-2").await?;
//! ```

mod catalog;
mod formats;
mod manager;

pub use catalog::{Dataset, DatasetCatalog, DatasetCategory, DatasetTier};
pub use formats::{CodeFormat, DocumentFormat, FormatHandler, FormatType, MediaFormat, MediaInfo};
pub use manager::{CacheEntry, DatasetConfig, DatasetFiles, DatasetManager, DownloadProgress};
