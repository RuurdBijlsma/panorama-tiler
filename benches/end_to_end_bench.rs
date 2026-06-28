use criterion::{Criterion, criterion_group, criterion_main};
use pano_tiler::{OutputConfig, PanoAngles, TilerConfig, process_panorama, save_to_disk};
use std::fs;
use std::path::Path;
use std::time::Duration;

fn bench_end_to_end_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("End-to-End Workload");

    // Disk I/O benchmarks have high latency and variance.
    // We use a small sample size to prevent the benchmark from taking too long.
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(280));
    group.warm_up_time(Duration::from_secs(5));

    let input_path = Path::new("img/sphere/PXL_20220918_115954889.PHOTOSPHERE.jpg");
    let output_dir = Path::new("target/bench_end_to_end_out");

    // Default configuration matching the Python script's defaults
    let config = TilerConfig {
        angles: PanoAngles::default(),
        output: OutputConfig {
            tile_size: 512,
            fallback_size: 1024,
            cube_size: 0,
            ..Default::default()
        },
    };

    group.bench_function("rust_native_pipeline", |b| {
        b.iter(|| {
            // 1. Clean up any previous run's directory to ensure accurate disk-write metrics
            if output_dir.exists() {
                let _ = fs::remove_dir_all(output_dir);
            }

            // 2. Read and decode the image from disk (matching Python PIL's image load step)
            let dynamic_img = image::open(input_path).expect("Failed to open benchmark image");
            let rgb_img = dynamic_img.to_rgb8();

            // 3. Process the panorama natively (calculating angles, cubemaps, and tile structures)
            let pano_output = process_panorama(&rgb_img, &config)
                .expect("Failed to process panorama in the tiler pipeline");

            // 4. Save tiles, fallback images, and config.json to disk
            save_to_disk(
                &pano_output,
                output_dir,
                config.output.format,
                config.output.quality,
            )
            .expect("Failed to save tiles to disk");
        })
    });

    // Final sweep to remove the output files left behind by the last iteration
    if output_dir.exists() {
        let _ = fs::remove_dir_all(output_dir);
    }

    group.finish();
}

criterion_group!(benches, bench_end_to_end_pipeline);
criterion_main!(benches);
