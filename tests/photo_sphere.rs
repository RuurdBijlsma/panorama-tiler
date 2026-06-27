use pano_tiler::{PartialPanoConfig, Projection, TilerConfig, process_panorama, save_to_disk};
use std::path::Path;

#[test]
fn test_generate_multires_panorama() {
    let img_path = Path::new("img/PXL_20220918_115954889.PHOTOSPHERE.jpg");
    assert!(img_path.exists());

    // Load the photosphere
    let dynamic_img = image::open(img_path).expect("Failed to open source integration test image");
    let rgb_img = dynamic_img.to_rgb8();

    // Define custom options (Defaults here model a full 360 panorama)
    let config = TilerConfig {
        projection: Projection::Equirectangular,
        partial_config: PartialPanoConfig::default(),
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
    let output_dir = Path::new("target/sphere_output");
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
