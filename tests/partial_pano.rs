use pano_tiler::config_helper::calculate_cylindrical_pano_config;
use pano_tiler::{PartialPanoConfig, Projection, TilerConfig, process_panorama, save_to_disk};
use std::path::Path;

#[test]
fn test_generate_multires_panorama() {
    let img_path = Path::new("img/PXL_20260414_113245071.PANO.jpg");
    assert!(img_path.exists());

    // Load the pano
    let dynamic_img = image::open(img_path).expect("Failed to open source integration test image");
    let rgb_img = dynamic_img.to_rgb8();

    // From exif:
    let width = 11136;
    let height = 3296;
    let focal_length_35mm_eq = 24.0;
    let crop_factor = 0.9; // Pano stitch crop factor
    let pano_config =
        calculate_cylindrical_pano_config(focal_length_35mm_eq, width, height, crop_factor)
            .unwrap();
    let config = TilerConfig {
        projection: Projection::Cylindrical,
        partial_config: pano_config,
        tile_size: 512,
        fallback_size: 1024,
        cube_size: 0,
        auto_load: true,
        png_output: false,
        quality: 75,
    };

    // Process the panorama
    let (tiles, config_json, actual_cube_size) = process_panorama(&rgb_img, &config)
        .expect("Failed to process panorama in the tiler pipeline");

    println!("Detected cube face resolution: {}", actual_cube_size);
    println!("Total zoom level hierarchy: {}", tiles.levels);
    println!("Total tiles generated: {}", tiles.tiles.len());

    // Save to disk
    let output_dir = Path::new("target/pano_output");
    if output_dir.exists() {
        std::fs::remove_dir_all(output_dir).expect("Failed to remove test output directory");
    }
    save_to_disk(
        &tiles,
        &config_json,
        output_dir,
        config.png_output,
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
