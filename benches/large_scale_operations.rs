use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use embeddenator::{EmbrFS, ReversibleVSAConfig};
use humansize::{format_size, DECIMAL};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Large-scale operations benchmark for 20GB-40GB datasets
///
/// Tests end-to-end performance of ingestion, extraction, and querying
/// on datasets that exceed typical RAM capacity.
#[cfg(feature = "large-scale")]
fn bench_large_scale_ingestion(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_scale_ingestion");
    group.sample_size(10); // Fewer samples for very large benchmarks
    group.measurement_time(Duration::from_secs(60)); // Allow longer measurement time

    // Test different dataset scales
    let scales = vec![
        ("5GB", 5 * 1024 * 1024 * 1024u64),
        ("10GB", 10 * 1024 * 1024 * 1024u64),
        ("20GB", 20 * 1024 * 1024 * 1024u64),
    ];

    for (label, target_size) in scales {
        group.bench_with_input(
            BenchmarkId::new("ingestion_throughput", label),
            &target_size,
            |bencher, &target_size| {
                bencher.iter_with_setup(
                    || create_large_test_dataset(target_size),
                    |temp_dir| {
                        let config = ReversibleVSAConfig::default();
                        let mut fs = EmbrFS::new();

                        let start = Instant::now();
                        let result = fs.ingest_directory(temp_dir.path(), false, &config);
                        let duration = start.elapsed();

                        // Calculate throughput
                        let throughput = target_size as f64 / duration.as_secs_f64();

                        println!(
                            "{} ingestion: {:.2} MB/s",
                            label,
                            throughput / (1024.0 * 1024.0)
                        );

                        black_box(result).unwrap()
                    },
                );
            },
        );
    }

    group.finish();
}

/// Create a large test dataset with realistic file distribution
fn create_large_test_dataset(target_size: u64) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    println!(
        "Creating {} test dataset...",
        format_size(target_size, DECIMAL)
    );
    let pb = ProgressBar::new(target_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut total_size = 0u64;
    let mut file_count = 0u32;

    // Create varied file types and sizes
    let file_types = vec![
        ".txt", ".json", ".bin", ".log", ".md", ".rs", ".py", ".js", ".html", ".css",
    ];

    while total_size < target_size {
        // Create subdirectories for realistic structure
        let subdir_depth = (file_count % 10) as usize;
        let mut current_path = base_path.to_path_buf();
        for depth in 0..subdir_depth {
            current_path = current_path.join(format!("level_{}", depth));
        }
        fs::create_dir_all(&current_path).unwrap();

        // Determine file size (1KB to 100MB, biased toward smaller files)
        let size_weights = vec![100, 50, 20, 10, 5, 2, 1, 1, 1, 1]; // Bias toward smaller files
        let size_options = vec![
            1024,
            10 * 1024,
            100 * 1024,
            1024 * 1024,
            10 * 1024 * 1024,
            50 * 1024 * 1024,
            100 * 1024 * 1024,
        ];
        let size_idx = (file_count as usize) % size_weights.len();
        let size = size_options[size_weights[size_idx] % size_options.len()];

        // Ensure we don't exceed target
        let actual_size = if total_size + size as u64 > target_size {
            (target_size - total_size) as usize
        } else {
            size
        };

        // Create filename
        let file_type = file_types[file_count as usize % file_types.len()];
        let filename = format!("file_{:08}{}", file_count, file_type);
        let file_path = current_path.join(filename);

        // Generate content based on file type
        let content = generate_realistic_content(file_type, actual_size);

        // Write file
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(&content).unwrap();

        total_size += actual_size as u64;
        file_count += 1;

        // Update progress bar
        pb.set_position(total_size.min(target_size));

        // Break if we've reached the target
        if total_size >= target_size {
            break;
        }
    }

    pb.finish_with_message(format!("Created {} files", file_count));
    temp_dir
}

/// Generate realistic content based on file type
fn generate_realistic_content(file_type: &str, size: usize) -> Vec<u8> {
    match file_type {
        ".txt" => {
            let lorem = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. ";
            let repeats = size / lorem.len();
            let remainder = size % lorem.len();
            let mut content = lorem.repeat(repeats);
            content.extend_from_slice(&lorem[..remainder]);
            content
        }
        ".json" => {
            let template =
                br#"{"data": "sample content", "size": 123, "nested": {"key": "value"}}"#;
            let repeats = size / template.len();
            let remainder = size % template.len();
            let mut content = template.repeat(repeats);
            content.extend_from_slice(&template[..remainder]);
            content
        }
        ".log" => {
            let log_line = b"[2024-01-11 12:00:00] INFO: Sample log message with some data\n";
            let repeats = size / log_line.len();
            let remainder = size % log_line.len();
            let mut content = log_line.repeat(repeats);
            content.extend_from_slice(&log_line[..remainder]);
            content
        }
        _ => {
            // For binary files, use pseudo-random but deterministic content
            (0..size).map(|i| ((i * 7 + 13) % 256) as u8).collect()
        }
    }
}

/// Benchmark extraction performance on large datasets
#[cfg(feature = "large-scale")]
fn bench_large_scale_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_scale_extraction");
    group.sample_size(5); // Very few samples for large benchmarks
    group.measurement_time(Duration::from_secs(120)); // Allow 2 minutes per sample

    let scales = vec![
        ("5GB", 5 * 1024 * 1024 * 1024u64),
        ("10GB", 10 * 1024 * 1024 * 1024u64),
    ];

    for (label, target_size) in scales {
        group.bench_with_input(
            BenchmarkId::new("extraction_throughput", label),
            &target_size,
            |bencher, &target_size| {
                bencher.iter_with_setup(
                    || {
                        // Create dataset and ingest it once
                        let temp_dir = create_large_test_dataset(target_size);
                        let config = ReversibleVSAConfig::default();
                        let mut fs = EmbrFS::new();
                        fs.ingest_directory(temp_dir.path(), false, &config)
                            .unwrap();

                        // Create extraction directory
                        let extract_dir = TempDir::new().unwrap();

                        (fs, temp_dir, extract_dir, config)
                    },
                    |(fs, _temp_dir, extract_dir, config)| {
                        let start = Instant::now();
                        let result = fs.extract_all_to_directory(extract_dir.path(), &config);
                        let duration = start.elapsed();

                        let throughput = target_size as f64 / duration.as_secs_f64();
                        println!(
                            "{} extraction: {:.2} MB/s",
                            label,
                            throughput / (1024.0 * 1024.0)
                        );

                        black_box(result).unwrap()
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage patterns during large operations
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    // Test memory scaling with different chunking strategies
    let chunk_sizes = vec![4096, 16384, 65536, 262144]; // 4KB to 256KB

    for chunk_size in chunk_sizes {
        group.bench_with_input(
            BenchmarkId::new("chunking_memory", format!("{}KB", chunk_size / 1024)),
            &chunk_size,
            |bencher, &chunk_size| {
                bencher.iter_with_setup(
                    || {
                        let config = ReversibleVSAConfig::default();
                        // Create a moderately large test file
                        let data = vec![0u8; 100 * 1024 * 1024]; // 100MB
                        SparseVec::encode_data(&data, &config, None)
                    },
                    |vec| {
                        // Simulate chunked processing
                        let chunks = vec.chunks(chunk_size);
                        let mut results = Vec::new();

                        for chunk in chunks {
                            let chunk_vec = SparseVec::encode_data(
                                chunk,
                                &ReversibleVSAConfig::default(),
                                None,
                            );
                            results.push(chunk_vec);
                        }

                        black_box(results)
                    },
                );
            },
        );
    }

    group.finish();
}

#[cfg(feature = "large-scale")]
criterion_group!(
    benches,
    bench_large_scale_ingestion,
    bench_large_scale_extraction,
    bench_memory_patterns
);

#[cfg(not(feature = "large-scale"))]
criterion_group!(benches, bench_memory_patterns);

criterion_main!(benches);
