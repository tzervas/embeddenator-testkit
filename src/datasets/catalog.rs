//! Dataset catalog defining available benchmark datasets
//!
//! Curated list of publicly available datasets for VSA testing,
//! organized by size tier and content category.

use serde::{Deserialize, Serialize};

/// Size tier for datasets - determines when they're used in testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DatasetTier {
    /// < 100MB - Used in CI/PR tests
    Quick,
    /// 100MB - 1GB - Daily integration tests
    Integration,
    /// 1GB - 10GB - Nightly benchmarks
    Nightly,
    /// 10GB - 20GB - Release validation
    Release,
}

impl DatasetTier {
    /// Maximum size in bytes for this tier
    pub fn max_size(&self) -> u64 {
        match self {
            DatasetTier::Quick => 100 * 1024 * 1024,         // 100 MB
            DatasetTier::Integration => 1024 * 1024 * 1024,  // 1 GB
            DatasetTier::Nightly => 10 * 1024 * 1024 * 1024, // 10 GB
            DatasetTier::Release => 20 * 1024 * 1024 * 1024, // 20 GB
        }
    }
}

/// Content category for datasets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DatasetCategory {
    /// Plain text documents (Wikipedia, books, articles)
    Text,
    /// Structured data (JSON, CSV, XML)
    Structured,
    /// Source code (multi-language)
    Code,
    /// Images (photos, diagrams, screenshots)
    Image,
    /// Video content
    Video,
    /// Audio content (music, speech, podcasts)
    Audio,
    /// Mixed document formats (PDF, DOCX, HTML)
    Document,
    /// Information retrieval benchmarks (MS MARCO, BEIR)
    Retrieval,
}

/// A downloadable benchmark dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    /// Unique identifier for the dataset
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of the dataset
    pub description: String,
    /// Size tier
    pub tier: DatasetTier,
    /// Content category
    pub category: DatasetCategory,
    /// Download URL (direct link or mirror)
    pub url: String,
    /// Expected SHA256 hash of the downloaded file
    pub sha256: Option<String>,
    /// Compressed size in bytes
    pub compressed_size: u64,
    /// Uncompressed size in bytes (approximate)
    pub uncompressed_size: u64,
    /// License identifier (SPDX)
    pub license: String,
    /// File formats contained in the dataset
    pub formats: Vec<String>,
    /// Whether the dataset requires extraction
    pub needs_extraction: bool,
    /// Compression format (gz, zst, zip, tar.gz, etc.)
    pub compression: Option<String>,
}

/// Catalog of all available datasets
pub struct DatasetCatalog {
    datasets: Vec<Dataset>,
}

impl Default for DatasetCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl DatasetCatalog {
    /// Create a new catalog with all known datasets
    pub fn new() -> Self {
        Self {
            datasets: Self::build_catalog(),
        }
    }

    /// Get all datasets
    pub fn all(&self) -> &[Dataset] {
        &self.datasets
    }

    /// Get datasets by tier
    pub fn by_tier(&self, tier: DatasetTier) -> Vec<&Dataset> {
        self.datasets.iter().filter(|d| d.tier == tier).collect()
    }

    /// Get datasets by category
    pub fn by_category(&self, category: DatasetCategory) -> Vec<&Dataset> {
        self.datasets
            .iter()
            .filter(|d| d.category == category)
            .collect()
    }

    /// Get a specific dataset by ID
    pub fn get(&self, id: &str) -> Option<&Dataset> {
        self.datasets.iter().find(|d| d.id == id)
    }

    /// Get datasets up to and including a tier (for cumulative testing)
    pub fn up_to_tier(&self, tier: DatasetTier) -> Vec<&Dataset> {
        let max_size = tier.max_size();
        self.datasets
            .iter()
            .filter(|d| d.compressed_size <= max_size)
            .collect()
    }

    fn build_catalog() -> Vec<Dataset> {
        vec![
            // ==================== QUICK TIER (<100MB) ====================

            // Text datasets
            Dataset {
                id: "wikitext-2".into(),
                name: "WikiText-2".into(),
                description: "Small Wikipedia extract for language modeling".into(),
                tier: DatasetTier::Quick,
                category: DatasetCategory::Text,
                url: "https://huggingface.co/datasets/Salesforce/wikitext/resolve/main/wikitext-2-raw-v1.zip".into(),
                sha256: None,
                compressed_size: 4_500_000,  // ~4.5 MB
                uncompressed_size: 11_000_000,  // ~11 MB
                license: "CC-BY-SA-3.0".into(),
                formats: vec!["txt".into()],
                needs_extraction: true,
                compression: Some("zip".into()),
            },
            Dataset {
                id: "20newsgroups".into(),
                name: "20 Newsgroups".into(),
                description: "Classic text classification dataset with newsgroup posts".into(),
                tier: DatasetTier::Quick,
                category: DatasetCategory::Text,
                url: "https://archive.ics.uci.edu/ml/machine-learning-databases/20newsgroups-mld/20news-bydate.tar.gz".into(),
                sha256: None,
                compressed_size: 15_000_000,  // ~15 MB
                uncompressed_size: 60_000_000,  // ~60 MB
                license: "Public Domain".into(),
                formats: vec!["txt".into()],
                needs_extraction: true,
                compression: Some("tar.gz".into()),
            },

            // Structured data
            Dataset {
                id: "json-sample-small".into(),
                name: "JSON Sample Collection".into(),
                description: "Diverse JSON documents for format testing".into(),
                tier: DatasetTier::Quick,
                category: DatasetCategory::Structured,
                url: "https://raw.githubusercontent.com/json-iterator/test-data/master/large-file.json".into(),
                sha256: None,
                compressed_size: 25_000_000,  // ~25 MB
                uncompressed_size: 25_000_000,
                license: "MIT".into(),
                formats: vec!["json".into()],
                needs_extraction: false,
                compression: None,
            },

            // Code datasets
            Dataset {
                id: "rosetta-code-small".into(),
                name: "Rosetta Code Samples".into(),
                description: "Multi-language code samples from Rosetta Code".into(),
                tier: DatasetTier::Quick,
                category: DatasetCategory::Code,
                url: "https://github.com/acmeism/RosettaCodeData/archive/refs/heads/master.zip".into(),
                sha256: None,
                compressed_size: 50_000_000,  // ~50 MB
                uncompressed_size: 200_000_000,  // ~200 MB
                license: "GFDL".into(),
                formats: vec!["rs".into(), "py".into(), "js".into(), "c".into(), "cpp".into(), "java".into()],
                needs_extraction: true,
                compression: Some("zip".into()),
            },

            // Image datasets
            Dataset {
                id: "cifar-10".into(),
                name: "CIFAR-10".into(),
                description: "60,000 32x32 color images in 10 classes".into(),
                tier: DatasetTier::Integration,  // 170MB exceeds Quick tier limit
                category: DatasetCategory::Image,
                url: "https://www.cs.toronto.edu/~kriz/cifar-10-binary.tar.gz".into(),
                sha256: None,
                compressed_size: 170_000_000,  // ~170 MB
                uncompressed_size: 180_000_000,
                license: "MIT".into(),
                formats: vec!["bin".into()],  // Binary format, but represents images
                needs_extraction: true,
                compression: Some("tar.gz".into()),
            },
            Dataset {
                id: "sample-images-small".into(),
                name: "Sample Images Collection".into(),
                description: "Diverse image formats for testing (PNG, JPEG, WebP, GIF)".into(),
                tier: DatasetTier::Quick,
                category: DatasetCategory::Image,
                url: "https://github.com/recurser/exif-orientation-examples/archive/refs/heads/master.zip".into(),
                sha256: None,
                compressed_size: 5_000_000,  // ~5 MB
                uncompressed_size: 10_000_000,
                license: "MIT".into(),
                formats: vec!["jpg".into(), "png".into(), "gif".into()],
                needs_extraction: true,
                compression: Some("zip".into()),
            },

            // Audio samples (small)
            Dataset {
                id: "freesound-samples".into(),
                name: "Free Sound Effects".into(),
                description: "Public domain sound effects and audio samples".into(),
                tier: DatasetTier::Quick,
                category: DatasetCategory::Audio,
                url: "https://freesound.org/data/previews/".into(),  // Note: Would need actual dataset URL
                sha256: None,
                compressed_size: 50_000_000,  // ~50 MB
                uncompressed_size: 80_000_000,
                license: "CC0".into(),
                formats: vec!["mp3".into(), "wav".into(), "flac".into()],
                needs_extraction: false,
                compression: None,
            },

            // ==================== INTEGRATION TIER (100MB - 1GB) ====================

            // Text datasets
            Dataset {
                id: "wikitext-103".into(),
                name: "WikiText-103".into(),
                description: "Large Wikipedia extract for language modeling".into(),
                tier: DatasetTier::Integration,
                category: DatasetCategory::Text,
                url: "https://huggingface.co/datasets/Salesforce/wikitext/resolve/main/wikitext-103-raw-v1.zip".into(),
                sha256: None,
                compressed_size: 190_000_000,  // ~190 MB
                uncompressed_size: 520_000_000,  // ~520 MB
                license: "CC-BY-SA-3.0".into(),
                formats: vec!["txt".into()],
                needs_extraction: true,
                compression: Some("zip".into()),
            },

            // Retrieval benchmarks
            Dataset {
                id: "msmarco-passage-small".into(),
                name: "MS MARCO Passage (Small)".into(),
                description: "Subset of MS MARCO passage ranking dataset".into(),
                tier: DatasetTier::Integration,
                category: DatasetCategory::Retrieval,
                url: "https://msmarco.blob.core.windows.net/msmarcoranking/collection.tar.gz".into(),
                sha256: None,
                compressed_size: 800_000_000,  // ~800 MB
                uncompressed_size: 2_500_000_000,  // ~2.5 GB (full, but we can subset)
                license: "MIT".into(),
                formats: vec!["tsv".into(), "txt".into()],
                needs_extraction: true,
                compression: Some("tar.gz".into()),
            },

            // Code datasets
            Dataset {
                id: "codesearchnet-small".into(),
                name: "CodeSearchNet (Python subset)".into(),
                description: "Code-documentation pairs for Python".into(),
                tier: DatasetTier::Integration,
                category: DatasetCategory::Code,
                url: "https://s3.amazonaws.com/code-search-net/CodeSearchNet/v2/python.zip".into(),
                sha256: None,
                compressed_size: 450_000_000,  // ~450 MB
                uncompressed_size: 1_200_000_000,
                license: "MIT".into(),
                formats: vec!["py".into(), "jsonl".into()],
                needs_extraction: true,
                compression: Some("zip".into()),
            },

            // Image datasets
            Dataset {
                id: "imagenet-tiny".into(),
                name: "Tiny ImageNet".into(),
                description: "100,000 images across 200 classes".into(),
                tier: DatasetTier::Integration,
                category: DatasetCategory::Image,
                url: "http://cs231n.stanford.edu/tiny-imagenet-200.zip".into(),
                sha256: None,
                compressed_size: 240_000_000,  // ~240 MB
                uncompressed_size: 400_000_000,
                license: "ImageNet License".into(),
                formats: vec!["jpeg".into()],
                needs_extraction: true,
                compression: Some("zip".into()),
            },

            // Video datasets
            Dataset {
                id: "kinetics-mini".into(),
                name: "Kinetics Mini Samples".into(),
                description: "Sample video clips for action recognition testing".into(),
                tier: DatasetTier::Integration,
                category: DatasetCategory::Video,
                url: "https://storage.googleapis.com/deepmind-media/Datasets/kinetics700_2020.tar.gz".into(),
                sha256: None,
                compressed_size: 500_000_000,  // ~500 MB (sample subset)
                uncompressed_size: 800_000_000,
                license: "CC-BY-4.0".into(),
                formats: vec!["mp4".into(), "webm".into()],
                needs_extraction: true,
                compression: Some("tar.gz".into()),
            },

            // ==================== NIGHTLY TIER (1GB - 10GB) ====================

            // Text datasets
            Dataset {
                id: "simple-wikipedia".into(),
                name: "Simple English Wikipedia".into(),
                description: "Full Simple English Wikipedia dump".into(),
                tier: DatasetTier::Nightly,
                category: DatasetCategory::Text,
                url: "https://dumps.wikimedia.org/simplewiki/latest/simplewiki-latest-pages-articles.xml.bz2".into(),
                sha256: None,
                compressed_size: 200_000_000,  // ~200 MB compressed
                uncompressed_size: 1_000_000_000,  // ~1 GB
                license: "CC-BY-SA-3.0".into(),
                formats: vec!["xml".into()],
                needs_extraction: true,
                compression: Some("bz2".into()),
            },

            // Code datasets
            Dataset {
                id: "codesearchnet-full".into(),
                name: "CodeSearchNet Full".into(),
                description: "Complete CodeSearchNet dataset (all languages)".into(),
                tier: DatasetTier::Nightly,
                category: DatasetCategory::Code,
                url: "https://s3.amazonaws.com/code-search-net/CodeSearchNet/v2/".into(),
                sha256: None,
                compressed_size: 6_000_000_000,  // ~6 GB
                uncompressed_size: 20_000_000_000,
                license: "MIT".into(),
                formats: vec!["py".into(), "js".into(), "go".into(), "java".into(), "php".into(), "ruby".into(), "jsonl".into()],
                needs_extraction: true,
                compression: Some("zip".into()),
            },

            // Retrieval benchmarks
            Dataset {
                id: "msmarco-full".into(),
                name: "MS MARCO Full".into(),
                description: "Complete MS MARCO passage and document ranking".into(),
                tier: DatasetTier::Nightly,
                category: DatasetCategory::Retrieval,
                url: "https://msmarco.blob.core.windows.net/msmarcoranking/".into(),
                sha256: None,
                compressed_size: 4_000_000_000,  // ~4 GB
                uncompressed_size: 12_000_000_000,
                license: "MIT".into(),
                formats: vec!["tsv".into(), "txt".into(), "jsonl".into()],
                needs_extraction: true,
                compression: Some("tar.gz".into()),
            },

            // Video datasets
            Dataset {
                id: "ucf101".into(),
                name: "UCF101 Action Recognition".into(),
                description: "13,320 video clips across 101 action categories".into(),
                tier: DatasetTier::Nightly,
                category: DatasetCategory::Video,
                url: "https://www.crcv.ucf.edu/data/UCF101/UCF101.rar".into(),
                sha256: None,
                compressed_size: 6_500_000_000,  // ~6.5 GB
                uncompressed_size: 7_000_000_000,
                license: "UCF License".into(),
                formats: vec!["avi".into()],
                needs_extraction: true,
                compression: Some("rar".into()),
            },

            // ==================== RELEASE TIER (10GB - 20GB) ====================

            // Text datasets
            Dataset {
                id: "wikipedia-en".into(),
                name: "English Wikipedia".into(),
                description: "Full English Wikipedia dump".into(),
                tier: DatasetTier::Release,
                category: DatasetCategory::Text,
                url: "https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-pages-articles.xml.bz2".into(),
                sha256: None,
                compressed_size: 20_000_000_000,  // ~20 GB compressed
                uncompressed_size: 80_000_000_000,  // ~80 GB
                license: "CC-BY-SA-3.0".into(),
                formats: vec!["xml".into()],
                needs_extraction: true,
                compression: Some("bz2".into()),
            },

            // Large text corpus
            Dataset {
                id: "openwebtext".into(),
                name: "OpenWebText".into(),
                description: "Open-source recreation of WebText corpus".into(),
                tier: DatasetTier::Release,
                category: DatasetCategory::Text,
                url: "https://skylion007.github.io/OpenWebTextCorpus/".into(),
                sha256: None,
                compressed_size: 12_000_000_000,  // ~12 GB
                uncompressed_size: 40_000_000_000,
                license: "CC0".into(),
                formats: vec!["txt".into()],
                needs_extraction: true,
                compression: Some("xz".into()),
            },

            // Image datasets
            Dataset {
                id: "imagenet-subset".into(),
                name: "ImageNet Subset (10%)".into(),
                description: "10% sample of ImageNet for validation".into(),
                tier: DatasetTier::Release,
                category: DatasetCategory::Image,
                url: "https://image-net.org/data/".into(),  // Requires registration
                sha256: None,
                compressed_size: 15_000_000_000,  // ~15 GB
                uncompressed_size: 20_000_000_000,
                license: "ImageNet License".into(),
                formats: vec!["jpeg".into()],
                needs_extraction: true,
                compression: Some("tar".into()),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_creation() {
        let catalog = DatasetCatalog::new();
        assert!(!catalog.all().is_empty());
    }

    #[test]
    fn test_tier_filtering() {
        let catalog = DatasetCatalog::new();
        let quick = catalog.by_tier(DatasetTier::Quick);
        assert!(!quick.is_empty());
        for ds in quick {
            assert!(ds.compressed_size <= DatasetTier::Quick.max_size());
        }
    }

    #[test]
    fn test_category_filtering() {
        let catalog = DatasetCatalog::new();
        let images = catalog.by_category(DatasetCategory::Image);
        assert!(!images.is_empty());
        for ds in images {
            assert_eq!(ds.category, DatasetCategory::Image);
        }
    }
}
