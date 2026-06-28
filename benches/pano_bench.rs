use criterion::{Criterion, criterion_group, criterion_main};
use image::{Rgb, RgbImage};
use pano_tiler::{
    PartialPanoConfig, Projection, TilerConfig, b83, calculate_pano_angles, projection, tiler,
};
use std::hint::black_box;
use std::time::Duration;

/// Generates a synthetic gradient equirectangular panorama in-memory
/// to avoid disk I/O noise during benchmarks.
fn generate_synthetic_pano(width: u32, height: u32) -> RgbImage {
    RgbImage::from_fn(width, height, |x, y| {
        let r = (x as f64 / width as f64 * 255.0) as u8;
        let g = (y as f64 / height as f64 * 255.0) as u8;
        let b = 128;
        Rgb([r, g, b])
    })
}

/// Benchmark Base83 encoding performance.
fn bench_base83(c: &mut Criterion) {
    let mut group = c.benchmark_group("Base83 Encoding");
    // Benchmark single values
    group.bench_function("encode_single_val", |b| {
        b.iter(|| b83::encode(black_box(&[82]), black_box(1)))
    });
    // Benchmark larger arrays (such as missing tiles coordinate sequences)
    let coordinates = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
    group.bench_function("encode_array", |b| {
        b.iter(|| b83::encode(black_box(&coordinates), black_box(2)))
    });
    group.finish();
}

/// Benchmark the math functions for calculating AOV angles.
fn bench_pano_angles(c: &mut Criterion) {
    c.bench_function("calculate_pano_angles", |b| {
        b.iter(|| {
            calculate_pano_angles(
                black_box(24.0), // 35mm equivalent
                black_box(6000), // width
                black_box(3000), // height
                black_box(0.90), // crop factor
            )
        })
    });
}

/// Benchmark cubemap face generation (Bicubic mapping + Rayon concurrency).
fn bench_cube_face_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Projection");
    // We use a small-to-moderate size for quick but meaningful benchmark runs.
    let src_image = generate_synthetic_pano(1024, 512);

    let config = TilerConfig {
        projection: Projection::Equirectangular,
        partial_config: PartialPanoConfig::default(),
        tile_size: 256,
        fallback_size: 0,
        cube_size: 256,
        auto_load: false,
        output_format: Default::default(),
        quality: 75,
    };

    group.bench_function("generate_cube_faces_256px", |b| {
        b.iter(|| {
            projection::generate_cube_faces(
                black_box(&src_image),
                black_box(&config),
                black_box(256), // target face resolution
            )
        })
    });

    group.finish();
}

/// Benchmark tiling, recursive downscaling (Lanczos3), and pyramid building.
fn bench_tiler_pyramid(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tiler");

    // Generate pre-computed faces to isolate the pyramid subdivision step
    let src_image = generate_synthetic_pano(512, 256);
    let config = TilerConfig {
        projection: Projection::Equirectangular,
        partial_config: PartialPanoConfig::default(),
        tile_size: 256,
        fallback_size: 512,
        cube_size: 256,
        auto_load: false,
        output_format: Default::default(),
        quality: 75,
    };

    let faces = projection::generate_cube_faces(&src_image, &config, 256);

    group.bench_function("generate_pyramid", |b| {
        b.iter(|| tiler::generate_pyramid(black_box(&faces), black_box(&config), black_box(256)))
    });

    group.finish();
}

fn bench_full_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("Full Pipeline");
    group.measurement_time(Duration::from_secs(30));
    group.warm_up_time(Duration::from_secs(10));
    group.sample_size(100);

    let src_image = generate_synthetic_pano(6000, 3000);
    let config = TilerConfig {
        projection: Projection::Equirectangular,
        partial_config: PartialPanoConfig::default(),
        tile_size: 512,
        fallback_size: 1024,
        cube_size: 0,
        auto_load: true,
        output_format: Default::default(),
        quality: 75,
    };

    group.bench_function("process_panorama", |b| {
        b.iter(|| pano_tiler::process_panorama(black_box(&src_image), black_box(&config)).unwrap())
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_base83,
    bench_pano_angles,
    bench_cube_face_generation,
    bench_tiler_pyramid,
    bench_full_integration,
);
criterion_main!(benches);
