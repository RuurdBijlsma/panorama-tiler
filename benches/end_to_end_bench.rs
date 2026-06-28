use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use panorama_tiler::{OutputConfig, tile_panorama_with_guessed_angles};
use std::fs;
use std::path::Path;
use std::time::Duration;

fn bench_end_to_end_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("End-to-End Workload");
    group.sample_size(30);
    group.sampling_mode(SamplingMode::Flat);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(35));

    let input_path = Path::new("img/sphere/PXL_20220918_115954889.PHOTOSPHERE.jpg");
    let output_dir = Path::new("target/bench_end_to_end_out");

    group.bench_function("rust_native_pipeline", |b| {
        b.iter(|| {
            // Clean up
            if output_dir.exists() {
                let _ = fs::remove_dir_all(output_dir);
            }

            // Process
            tile_panorama_with_guessed_angles(
                input_path,
                output_dir,
                Some(OutputConfig {
                    tile_size: 512,
                    fallback_size: 1024,
                    cube_size: 0,
                    ..Default::default()
                }),
            )
            .expect("Guessed-angles pipeline execution failed");
        })
    });

    // Clean up
    if output_dir.exists() {
        let _ = fs::remove_dir_all(output_dir);
    }

    group.finish();
}

criterion_group!(benches, bench_end_to_end_pipeline);
criterion_main!(benches);
