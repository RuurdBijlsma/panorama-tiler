use std::path::Path;
use pano_tiler::{TilerConfig, PartialPanoConfig, Projection, process_panorama, save_to_disk};

#[test]
fn test_generate_multires_panorama() {
    let img_path = Path::new("img/PXL_20220918_115954889.PHOTOSPHERE.jpg");

    // Skip the integration test if the input image is not present
    assert!(img_path.exists());

    // 1. Load the photosphere
    let dynamic_img = image::open(img_path)
        .expect("Failed to open source integration test image");
    let rgb_img = dynamic_img.to_rgb8();

    // 2. Define custom options (Defaults here model a full 360 panorama)
    let config = TilerConfig {
        // [XMP-GPano] ProjectionType : equirectangular
        projection: Projection::Equirectangular,

        partial_config: PartialPanoConfig {
            // Full panorama coverage
            haov: 360.0,
            vaov: 180.0,
            v_offset: 0.0,
            horizon_pixels: 0,
            background_color: [0.0, 0.0, 0.0],
            avoid_showing_background: false,
        },

        // Standard multires tile sizes (512 is standard for Pannellum)
        tile_size: 512,

        // Fallback cube size for older devices (set to 0 to disable)
        fallback_size: 1024,

        // Let the library auto-calculate the cube resolution from the 8704px width.
        // The formula (8704 / PI) will yield a cube face size of 2768px.
        cube_size: 0,

        auto_load: true,
        png_output: false, // Save as JPG (standard for photo panoramas)
        quality: 75,       // Output JPEG quality (75 is a balanced default)
    };

    // 3. Process the panorama inside the pure Rust pipeline
    let (tiles, config_json, actual_cube_size) = process_panorama(&rgb_img, &config)
        .expect("Failed to process panorama in the tiler pipeline");

    println!("Detected cube face resolution: {}", actual_cube_size);
    println!("Total zoom level hierarchy: {}", tiles.levels);
    println!("Total tiles generated: {}", tiles.tiles.len());

    // 4. Save everything to disk
    let output_dir = Path::new("target/test_output");
    if output_dir.exists(){
        std::fs::remove_dir_all(output_dir).expect("Failed to remove test output directory");
    }
    save_to_disk(&tiles, &config_json, output_dir, config.png_output, config.quality)
        .expect("Failed to save tiles and configuration json to target test folder");

    // 5. Basic verification assertions
    assert!(output_dir.join("config.json").exists(), "Missing config.json");
    assert!(output_dir.join("1").exists(), "Missing zoom level directory '1'");
    assert!(output_dir.join("fallback").exists(), "Missing fallback folder");
}