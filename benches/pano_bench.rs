use criterion::{Criterion, criterion_group, criterion_main};
use image::{Rgb, RgbImage};
use pano_tiler::{
    PartialPanoConfig, Projection, TilerConfig, b83, calculate_pano_angles, projection, tiler,
};
use std::hint::black_box;
use std::time::Duration;

fn generate_synthetic_pano(width: u32, height: u32) -> RgbImage {
    RgbImage::from_fn(width, height, |x, y| {
        let r = (x as f64 / width as f64 * 255.0) as u8;
        let g = (y as f64 / height as f64 * 255.0) as u8;
        let b = 128;
        Rgb([r, g, b])
    })
}

fn bench_base83(c: &mut Criterion) {
    let mut group = c.benchmark_group("Base83 Encoding");
    group.bench_function("encode_single_val", |b| {
        b.iter(|| b83::encode(black_box(&[82]), black_box(1)))
    });
    let coordinates = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
    group.bench_function("encode_array", |b| {
        b.iter(|| b83::encode(black_box(&coordinates), black_box(2)))
    });
    group.finish();
}

fn bench_pano_angles(c: &mut Criterion) {
    c.bench_function("calculate_pano_angles", |b| {
        b.iter(|| {
            calculate_pano_angles(
                black_box(24.0),
                black_box(6000),
                black_box(3000),
                black_box(0.90),
            )
        })
    });
}

/// Benchmark Cubemap generation across Equirectangular vs Cylindrical and Full vs Partial.
fn bench_cube_face_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Projection");
    let src_image = generate_synthetic_pano(1024, 512);

    // Full Equirectangular
    let config_full = TilerConfig {
        projection: Projection::Equirectangular,
        partial_config: PartialPanoConfig::default(),
        tile_size: 256,
        fallback_size: 0,
        cube_size: 256,
        auto_load: false,
        output_format: Default::default(),
        quality: 75,
    };
    group.bench_function("generate_cube_faces_equirect_full", |b| {
        b.iter(|| {
            projection::generate_cube_faces(
                black_box(&src_image),
                black_box(&config_full),
                black_box(256),
            )
        })
    });

    // 2. Partial Cylindrical
    let config_partial_cyl = TilerConfig {
        projection: Projection::Cylindrical,
        partial_config: PartialPanoConfig {
            haov: 180.0,
            vaov: 90.0,
            ..Default::default()
        },
        tile_size: 256,
        fallback_size: 0,
        cube_size: 256,
        auto_load: false,
        output_format: Default::default(),
        quality: 75,
    };
    group.bench_function("generate_cube_faces_cylindrical_partial", |b| {
        b.iter(|| {
            projection::generate_cube_faces(
                black_box(&src_image),
                black_box(&config_partial_cyl),
                black_box(256),
            )
        })
    });

    group.finish();
}

/// Benchmark Tiling and Pyramid generation (Full vs Partial).
fn bench_tiler_pyramid(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tiler");
    let src_image = generate_synthetic_pano(512, 256);

    // Full configuration
    let config_full = TilerConfig {
        projection: Projection::Equirectangular,
        partial_config: PartialPanoConfig::default(),
        tile_size: 256,
        fallback_size: 512,
        cube_size: 256,
        auto_load: false,
        output_format: Default::default(),
        quality: 75,
    };
    let faces_full = projection::generate_cube_faces(&src_image, &config_full, 256);

    group.bench_function("generate_pyramid_full", |b| {
        b.iter(|| {
            tiler::generate_pyramid(
                black_box(&faces_full),
                black_box(&config_full),
                black_box(256),
            )
        })
    });

    // Partial configuration (forces background-check iteration and missing-tiles formatting)
    let config_partial = TilerConfig {
        projection: Projection::Equirectangular,
        partial_config: PartialPanoConfig {
            haov: 120.0,
            vaov: 60.0,
            background_color: [0.0, 0.0, 0.0],
            ..Default::default()
        },
        tile_size: 256,
        fallback_size: 512,
        cube_size: 256,
        auto_load: false,
        output_format: Default::default(),
        quality: 75,
    };
    let faces_partial = projection::generate_cube_faces(&src_image, &config_partial, 256);

    group.bench_function("generate_pyramid_partial", |b| {
        b.iter(|| {
            tiler::generate_pyramid(
                black_box(&faces_partial),
                black_box(&config_partial),
                black_box(256),
            )
        })
    });

    group.finish();
}

fn bench_full_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("Full Pipeline");
    group.sample_size(30);
    group.measurement_time(Duration::from_secs(40));
    group.warm_up_time(Duration::from_secs(5));

    let src_image = generate_synthetic_pano(4000, 2000);
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

    group.bench_function("process_panorama_4k", |b| {
        b.iter(|| {
            pano_tiler::process_panorama(black_box(&src_image), black_box(&config)).unwrap()
        })
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