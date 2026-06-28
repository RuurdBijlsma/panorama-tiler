use pano_tiler::{GeneratorConfig, OutputFormat, process_panorama, save_to_disk};
use std::path::Path;

#[test]
fn test_generate_multires_panorama() {
    let input_images = &[
        Path::new("img/sphere/PXL_20220918_115954889.PHOTOSPHERE.jpg"),
        Path::new("img/sphere/PXL_20210722_151141413.PHOTOSPHERE.jpg"),
    ];
    let out_formats = &[
        OutputFormat::Jpeg,
        OutputFormat::Png,
        #[cfg(feature = "webp")]
        OutputFormat::Webp,
    ];
    // webp 85 seems a good balance
    let qualities = &[85];
    for out_format in out_formats {
        for quality in qualities {
            for image_path in input_images {
                generate_pano(image_path, *out_format, *quality);
            }
        }
    }
}

fn generate_pano(image_path: &Path, output_format: OutputFormat, quality: u8) {
    assert!(image_path.exists());

    // Load the photosphere
    let dynamic_img =
        image::open(image_path).expect("Failed to open source integration test image");
    let rgb_img = dynamic_img.to_rgb8();

    // Define custom options (Defaults here model a full 360 panorama)
    let config = GeneratorConfig {
        output_format,
        quality,
        ..Default::default()
    };

    // Process the panorama
    let (tiles, config_json, actual_cube_size) = process_panorama(&rgb_img, &config)
        .expect("Failed to process panorama in the tiler pipeline");

    println!("Detected cube face resolution: {}", actual_cube_size);
    println!("Total zoom level hierarchy: {}", tiles.levels);
    println!("Total tiles generated: {}", tiles.tiles.len());

    // Save to disk
    let out_path = format!(
        "target/sphere_output_{}_q{}_{}",
        config.output_format.to_extension(),
        config.quality,
        image_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );
    let output_dir = Path::new(&out_path);
    if output_dir.exists() {
        std::fs::remove_dir_all(output_dir).expect("Failed to remove test output directory");
    }
    save_to_disk(
        &tiles,
        &config_json,
        output_dir,
        config.output_format,
        config.quality,
    )
    .expect("Failed to save tiles and configuration json to target test folder");

    // Assertions
    assert!(
        output_dir.join("config.json").exists(),
        "Missing config.json"
    );
    assert!(
        output_dir.join("1").exists(),
        "Missing zoom level directory '1'"
    );
    assert!(
        output_dir.join("fallback").exists(),
        "Missing fallback folder"
    );
}
