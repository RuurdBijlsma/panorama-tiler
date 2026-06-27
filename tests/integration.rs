use std::path::Path;
use pano_tiler::{TilerConfig, PartialPanoConfig, Projection, process_panorama, save_to_disk};

#[test]
fn test_generate_multires_panorama() {
    let img_path = Path::new("img/PXL_20220918_115954889.PHOTOSPHERE.jpg");

    // Skip the integration test if the input image is not present
    if !img_path.exists() {
        println!("Integration test skipped: image not found at {:?}", img_path);
        return;
    }

    // 1. Load the photosphere
    let dynamic_img = image::open(img_path)
        .expect("Failed to open source integration test image");
    let rgb_img = dynamic_img.to_rgb8();

    // 2. Define custom options (Defaults here model a full 360 panorama)
    let config = TilerConfig {
        projection: Projection::Equirectangular,
        partial_config: PartialPanoConfig {
            haov: -1.0,  // Auto-detect 360.0
            vaov: -1.0,  // Auto-detect 180.0
            ..Default::default()
        },
        tile_size: 512,
        fallback_size: 1024,
        cube_size: 0,    // 0 lets the library calculate size from the input width
        auto_load: true,
        png_output: false,
        quality: 75,
    };

    // 3. Process the panorama inside the pure Rust pipeline
    let (tiles, config_json, actual_cube_size) = process_panorama(&rgb_img, &config)
        .expect("Failed to process panorama in the tiler pipeline");

    println!("Detected cube face resolution: {}", actual_cube_size);
    println!("Total zoom level hierarchy: {}", tiles.levels);
    println!("Total tiles generated: {}", tiles.tiles.len());

    // 4. Save everything to disk
    let output_dir = Path::new("target/test_output");
    save_to_disk(&tiles, &config_json, output_dir, config.png_output, config.quality)
        .expect("Failed to save tiles and configuration json to target test folder");

    // 5. Basic verification assertions
    assert!(output_dir.join("config.json").exists(), "Missing config.json");
    assert!(output_dir.join("1").exists(), "Missing zoom level directory '1'");
    assert!(output_dir.join("fallback").exists(), "Missing fallback folder");
}