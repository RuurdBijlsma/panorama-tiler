use pano_tiler::{
    OutputFormat, PartialPanoConfig, Projection, TilerConfig, process_panorama, save_to_disk,
};
use std::path::Path;

#[test]
fn test_generate_multires_panorama() {
    let out_formats = &[OutputFormat::Jpeg, OutputFormat::Png, OutputFormat::Webp];
    let qualities = &[75, 85, 95];
    for out_format in out_formats {
        for quality in qualities {
            generate_pano(*out_format, *quality);
        }
    }
}

fn generate_pano(output_format: OutputFormat, quality: u8) {
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
        output_format,
        quality,
    };

    // Process the panorama
    let (tiles, config_json, actual_cube_size) = process_panorama(&rgb_img, &config)
        .expect("Failed to process panorama in the tiler pipeline");

    println!("Detected cube face resolution: {}", actual_cube_size);
    println!("Total zoom level hierarchy: {}", tiles.levels);
    println!("Total tiles generated: {}", tiles.tiles.len());

    // Save to disk
    let out_path = format!(
        "target/sphere_output_{}_q{}",
        config.output_format.to_extension(),
        config.quality
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
