use pano_tiler::exif_helper::{exif_to_partial_pano_config, PanoExif};
use pano_tiler::{PartialPanoConfig, GeneratorConfig, process_panorama, save_to_disk};
use std::path::Path;

#[test]
fn test_generate_multires_panorama() {
    let img_path = Path::new("img/PXL_20210730_144252204.PHOTOSPHERE.jpg");
    assert!(img_path.exists());

    // Load the pano
    let dynamic_img = image::open(img_path).expect("Failed to open source integration test image");
    let rgb_img = dynamic_img.to_rgb8();

    // From exif:
    let partial_config = exif_to_partial_pano_config(&PanoExif {
        cropped_area_image_height_pixels: 4530,
        cropped_area_image_width_pixels: 5890,
        full_pano_height_pixels: 7731,
        full_pano_width_pixels: 15462,
        cropped_area_top_pixels: 282,
        pose_heading_degrees: Some(135.0),
    });
    let config = GeneratorConfig {
        partial_config,
        avoid_showing_background: false,
        ..Default::default()
    };

    // Process the panorama
    let (tiles, config_json, actual_cube_size) = process_panorama(&rgb_img, &config)
        .expect("Failed to process panorama in the tiler pipeline");

    println!("Detected cube face resolution: {}", actual_cube_size);
    println!("Total zoom level hierarchy: {}", tiles.levels);
    println!("Total tiles generated: {}", tiles.tiles.len());

    // Save to disk
    let output_dir = Path::new("target/partial_sphere");
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
