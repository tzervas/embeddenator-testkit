//! Dataset download and cache management
//!
//! Provides functionality for:
//! - Downloading datasets from various sources
//! - Caching datasets locally
//! - Extracting compressed archives
//! - Progress reporting

use super::catalog::{Dataset, DatasetCatalog, DatasetTier};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Configuration for dataset management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetConfig {
    /// Base directory for dataset cache
    pub cache_dir: PathBuf,
    /// Maximum cache size in bytes (0 = unlimited)
    pub max_cache_size: u64,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
    /// Number of retry attempts for downloads
    pub retry_count: u32,
    /// Enable parallel downloads
    pub parallel_downloads: bool,
    /// Maximum concurrent downloads
    pub max_concurrent: usize,
}

impl Default for DatasetConfig {
    fn default() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("embeddenator")
            .join("datasets");

        Self {
            cache_dir,
            max_cache_size: 0, // Unlimited by default
            timeout_secs: 300, // 5 minutes
            retry_count: 3,
            parallel_downloads: true,
            max_concurrent: 4,
        }
    }
}

/// Status of a dataset in cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Dataset ID
    pub id: String,
    /// Path to cached data
    pub path: PathBuf,
    /// Size on disk in bytes
    pub size: u64,
    /// Download timestamp
    pub downloaded_at: chrono::DateTime<chrono::Utc>,
    /// Whether extraction is complete
    pub extracted: bool,
    /// Checksum verification status
    pub verified: bool,
}

/// Progress callback type for downloads
pub type ProgressCallback = Box<dyn Fn(DownloadProgress) + Send + Sync>;

/// Download progress information
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Dataset being downloaded
    pub dataset_id: String,
    /// Bytes downloaded so far
    pub downloaded: u64,
    /// Total bytes to download (if known)
    pub total: Option<u64>,
    /// Current download speed in bytes/sec
    pub speed: f64,
    /// Estimated time remaining in seconds
    pub eta_secs: Option<f64>,
}

/// Dataset manager for downloading and caching datasets
pub struct DatasetManager {
    config: DatasetConfig,
    catalog: DatasetCatalog,
    cache: HashMap<String, CacheEntry>,
}

impl DatasetManager {
    /// Create a new dataset manager with default configuration
    pub fn new() -> Self {
        Self::with_config(DatasetConfig::default())
    }

    /// Create a new dataset manager with custom configuration
    pub fn with_config(config: DatasetConfig) -> Self {
        let catalog = DatasetCatalog::new();
        Self {
            config,
            catalog,
            cache: HashMap::new(),
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &DatasetConfig {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut DatasetConfig {
        &mut self.config
    }

    /// Load cache index from disk
    pub fn load_cache(&mut self) -> anyhow::Result<()> {
        let index_path = self.config.cache_dir.join("cache_index.json");

        if index_path.exists() {
            let content = std::fs::read_to_string(&index_path)?;
            self.cache = serde_json::from_str(&content)?;

            // Verify entries still exist on disk
            self.cache.retain(|_id, entry| entry.path.exists());
        }

        Ok(())
    }

    /// Save cache index to disk
    pub fn save_cache(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.config.cache_dir)?;

        let index_path = self.config.cache_dir.join("cache_index.json");
        let content = serde_json::to_string_pretty(&self.cache)?;
        std::fs::write(&index_path, content)?;

        Ok(())
    }

    /// Get dataset by ID
    pub fn get_dataset(&self, id: &str) -> Option<&Dataset> {
        self.catalog.get(id)
    }

    /// List all available datasets
    pub fn list_datasets(&self) -> Vec<&Dataset> {
        self.catalog.all().iter().collect()
    }

    /// List datasets by tier
    pub fn list_by_tier(&self, tier: DatasetTier) -> Vec<&Dataset> {
        self.catalog.by_tier(tier)
    }

    /// Check if a dataset is cached
    pub fn is_cached(&self, id: &str) -> bool {
        self.cache.contains_key(id)
    }

    /// Get path to cached dataset
    pub fn cached_path(&self, id: &str) -> Option<&Path> {
        self.cache.get(id).map(|e| e.path.as_path())
    }

    /// Get cache entry for a dataset
    pub fn cache_entry(&self, id: &str) -> Option<&CacheEntry> {
        self.cache.get(id)
    }

    /// Calculate total cache size
    pub fn cache_size(&self) -> u64 {
        self.cache.values().map(|e| e.size).sum()
    }

    /// Clear entire cache
    pub fn clear_cache(&mut self) -> anyhow::Result<()> {
        for entry in self.cache.values() {
            if entry.path.exists() {
                if entry.path.is_dir() {
                    std::fs::remove_dir_all(&entry.path)?;
                } else {
                    std::fs::remove_file(&entry.path)?;
                }
            }
        }
        self.cache.clear();
        self.save_cache()?;
        Ok(())
    }

    /// Remove a specific dataset from cache
    pub fn remove_from_cache(&mut self, id: &str) -> anyhow::Result<()> {
        if let Some(entry) = self.cache.remove(id) {
            if entry.path.exists() {
                if entry.path.is_dir() {
                    std::fs::remove_dir_all(&entry.path)?;
                } else {
                    std::fs::remove_file(&entry.path)?;
                }
            }
            self.save_cache()?;
        }
        Ok(())
    }

    /// Download a dataset
    #[cfg(feature = "realworld-datasets")]
    pub async fn download(&mut self, id: &str) -> anyhow::Result<PathBuf> {
        self.download_with_progress(id, None).await
    }

    /// Download a dataset with progress callback
    #[cfg(feature = "realworld-datasets")]
    pub async fn download_with_progress(
        &mut self,
        id: &str,
        progress: Option<ProgressCallback>,
    ) -> anyhow::Result<PathBuf> {
        // Check if already cached
        if let Some(entry) = self.cache.get(id) {
            if entry.path.exists() && entry.verified {
                return Ok(entry.path.clone());
            }
        }

        // Get dataset info
        let dataset = self
            .catalog
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Dataset not found: {}", id))?
            .clone();

        // Create download directory
        let download_dir = self.config.cache_dir.join(&dataset.id);
        std::fs::create_dir_all(&download_dir)?;

        // Determine output filename
        let url = dataset.url.clone();
        let filename = url.split('/').last().unwrap_or("download");
        let archive_path = download_dir.join(filename);

        // Download with retries
        let mut last_error = None;
        for attempt in 0..=self.config.retry_count {
            if attempt > 0 {
                tracing::info!("Retry attempt {} for {}", attempt, id);
                tokio::time::sleep(tokio::time::Duration::from_secs(2u64.pow(attempt))).await;
            }

            match self.download_file(&url, &archive_path, id, &progress).await {
                Ok(()) => {
                    last_error = None;
                    break;
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        if let Some(e) = last_error {
            return Err(e);
        }

        // Verify checksum if provided
        let verified = if let Some(expected) = &dataset.sha256 {
            verify_sha256(&archive_path, expected)?
        } else {
            true
        };

        // Extract if needed
        let final_path = if Self::is_archive(&archive_path) {
            let extract_dir = download_dir.join("data");
            Self::extract_archive(&archive_path, &extract_dir)?;
            extract_dir
        } else {
            archive_path.clone()
        };

        // Update cache
        let size = calculate_size(&final_path)?;
        self.cache.insert(
            id.to_string(),
            CacheEntry {
                id: id.to_string(),
                path: final_path.clone(),
                size,
                downloaded_at: chrono::Utc::now(),
                extracted: Self::is_archive(&archive_path),
                verified,
            },
        );
        self.save_cache()?;

        Ok(final_path)
    }

    #[cfg(feature = "realworld-datasets")]
    async fn download_file(
        &self,
        url: &str,
        path: &Path,
        dataset_id: &str,
        progress: &Option<ProgressCallback>,
    ) -> anyhow::Result<()> {
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .build()?;

        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let total = response.content_length();
        let mut downloaded: u64 = 0;
        let start_time = std::time::Instant::now();

        let mut file = tokio::fs::File::create(path).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            if let Some(ref cb) = progress {
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 {
                    downloaded as f64 / elapsed
                } else {
                    0.0
                };
                let eta = total.map(|t| {
                    if speed > 0.0 {
                        (t - downloaded) as f64 / speed
                    } else {
                        f64::INFINITY
                    }
                });

                cb(DownloadProgress {
                    dataset_id: dataset_id.to_string(),
                    downloaded,
                    total,
                    speed,
                    eta_secs: eta,
                });
            }
        }

        file.flush().await?;
        Ok(())
    }

    fn is_archive(path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        matches!(
            ext.to_lowercase().as_str(),
            "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz"
        ) || path.to_string_lossy().contains(".tar.")
    }

    #[cfg(feature = "realworld-datasets")]
    fn extract_archive(archive: &Path, dest: &Path) -> anyhow::Result<()> {
        use std::fs::File;

        std::fs::create_dir_all(dest)?;

        let ext = archive.extension().and_then(|e| e.to_str()).unwrap_or("");

        let path_str = archive.to_string_lossy();

        if ext == "zip" {
            // Extract ZIP
            let file = File::open(archive)?;
            let mut zip = zip::ZipArchive::new(file)?;
            zip.extract(dest)?;
        } else if path_str.ends_with(".tar.gz") || ext == "tgz" {
            // Extract tar.gz
            let file = File::open(archive)?;
            let gz = flate2::read::GzDecoder::new(file);
            let mut tar = tar::Archive::new(gz);
            tar.unpack(dest)?;
        } else if ext == "tar" {
            // Extract plain tar
            let file = File::open(archive)?;
            let mut tar = tar::Archive::new(file);
            tar.unpack(dest)?;
        } else if ext == "gz" {
            // Extract gzip (single file)
            use std::io::{Read, Write};
            let file = File::open(archive)?;
            let mut gz = flate2::read::GzDecoder::new(file);
            let mut content = Vec::new();
            gz.read_to_end(&mut content)?;

            // Use original filename without .gz
            let stem = archive
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("extracted");
            let out_path = dest.join(stem);
            let mut out_file = File::create(out_path)?;
            out_file.write_all(&content)?;
        } else {
            return Err(anyhow::anyhow!("Unsupported archive format: {}", ext));
        }

        Ok(())
    }

    /// Stub when feature is not enabled
    #[cfg(not(feature = "realworld-datasets"))]
    pub async fn download(&mut self, _id: &str) -> anyhow::Result<PathBuf> {
        Err(anyhow::anyhow!(
            "Dataset downloads not enabled. Enable the 'realworld-datasets' feature."
        ))
    }

    /// Download all datasets in a tier
    #[cfg(feature = "realworld-datasets")]
    pub async fn download_tier(&mut self, tier: DatasetTier) -> anyhow::Result<Vec<PathBuf>> {
        let datasets: Vec<String> = self
            .catalog
            .by_tier(tier)
            .iter()
            .map(|d| d.id.clone())
            .collect();

        let mut paths = Vec::new();
        for id in datasets {
            let path = self.download(&id).await?;
            paths.push(path);
        }

        Ok(paths)
    }
}

impl Default for DatasetManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify SHA256 checksum of a file
#[cfg(feature = "realworld-datasets")]
fn verify_sha256(path: &Path, expected: &str) -> anyhow::Result<bool> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    let actual = hex::encode(result);

    if actual == expected.to_lowercase() {
        Ok(true)
    } else {
        tracing::warn!("Checksum mismatch: expected {}, got {}", expected, actual);
        Ok(false)
    }
}

/// Calculate size of a file or directory
fn calculate_size(path: &Path) -> anyhow::Result<u64> {
    if path.is_file() {
        Ok(std::fs::metadata(path)?.len())
    } else if path.is_dir() {
        let mut total = 0;
        for entry in walkdir::WalkDir::new(path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                total += entry.metadata()?.len();
            }
        }
        Ok(total)
    } else {
        Ok(0)
    }
}

/// Iterator over files in a cached dataset
pub struct DatasetFiles {
    walker: walkdir::IntoIter,
}

impl DatasetFiles {
    /// Create iterator over all files in a directory
    pub fn new(path: &Path) -> Self {
        Self {
            walker: walkdir::WalkDir::new(path).into_iter(),
        }
    }

    /// Create iterator with specific extensions filter
    pub fn with_extensions<'a>(
        path: &'a Path,
        extensions: &'a [&'a str],
    ) -> impl Iterator<Item = PathBuf> + 'a {
        walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(move |e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| extensions.iter().any(|&e| e.eq_ignore_ascii_case(ext)))
                    .unwrap_or(false)
            })
            .map(|e| e.into_path())
    }
}

impl Iterator for DatasetFiles {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.walker.next() {
                Some(Ok(entry)) if entry.file_type().is_file() => {
                    return Some(entry.into_path());
                }
                Some(Ok(_)) => continue, // Skip directories
                Some(Err(_)) => continue,
                None => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatasetConfig::default();
        assert!(config.cache_dir.to_string_lossy().contains("embeddenator"));
        assert_eq!(config.timeout_secs, 300);
        assert_eq!(config.retry_count, 3);
    }

    #[test]
    fn test_manager_creation() {
        let manager = DatasetManager::new();
        assert!(!manager.list_datasets().is_empty());
    }

    #[test]
    fn test_is_archive_detection() {
        assert!(DatasetManager::is_archive(Path::new("test.zip")));
        assert!(DatasetManager::is_archive(Path::new("test.tar.gz")));
        assert!(DatasetManager::is_archive(Path::new("test.tgz")));
        assert!(!DatasetManager::is_archive(Path::new("test.txt")));
    }
}
