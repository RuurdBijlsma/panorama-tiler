use panorama_tiler::exif::{PanoExif, exif_to_partial_pano_config};
use panorama_tiler::{OutputConfig, TilerConfig, process_panorama, save_to_disk};
use std::path::{Path, PathBuf};

fn image1_config() -> (PathBuf, TilerConfig) {
    let img = Path::new("img/partial_sphere/PXL_20210730_144252204.PHOTOSPHERE.jpg").to_path_buf();
    // From exif:
    let partial_config = exif_to_partial_pano_config(&PanoExif {
        cropped_area_image_height_pixels: 4530,
        cropped_area_image_width_pixels: 5890,
        full_pano_height_pixels: 7731,
        full_pano_width_pixels: 15462,
        cropped_area_top_pixels: 282,
        projection_type: Some("equirectangular".to_string()),
        pose_heading_degrees: Some(135.0),
    });
    let config = TilerConfig {
        angles: partial_config,
        ..TilerConfig::default()
    };
    (img, config)
}

fn image2_config() -> (PathBuf, TilerConfig) {
    let img = Path::new("img/partial_sphere/PXL_20210730_183041272.PANO.jpg").to_path_buf();
    // From exif:
    let partial_config = exif_to_partial_pano_config(&PanoExif {
        cropped_area_image_height_pixels: 1640,
        cropped_area_image_width_pixels: 4512,
        full_pano_height_pixels: 4653,
        full_pano_width_pixels: 9306,
        cropped_area_top_pixels: 1605,
        projection_type: Some("equirectangular".to_string()),
        pose_heading_degrees: Some(87.0),
    });
    let config = TilerConfig {
        angles: partial_config,
        output: OutputConfig {
            yaw_padding: 10.0,
            pitch_padding: 5.0,
            ..OutputConfig::default()
        },
    };
    (img, config)
}

fn image3_config() -> (PathBuf, TilerConfig) {
    let img = Path::new("img/partial_sphere/PANO_20210207_152515.jpg").to_path_buf();
    // From exif:
    let partial_config = exif_to_partial_pano_config(&PanoExif {
        cropped_area_image_height_pixels: 5878,
        cropped_area_image_width_pixels: 5442,
        full_pano_height_pixels: 6809,
        full_pano_width_pixels: 13617,
        cropped_area_top_pixels: 918,
        projection_type: Some("equirectangular".to_string()),
        pose_heading_degrees: Some(215.0),
    });
    let config = TilerConfig {
        angles: partial_config,
        ..TilerConfig::default()
    };
    (img, config)
}

fn image4_config() -> (PathBuf, TilerConfig) {
    let img = Path::new("img/partial_sphere/PANO_20200806_210426.jpg").to_path_buf();
    // From exif:
    let partial_config = exif_to_partial_pano_config(&PanoExif {
        cropped_area_image_height_pixels: 4582,
        cropped_area_image_width_pixels: 6982,
        full_pano_height_pixels: 6959,
        full_pano_width_pixels: 13918,
        cropped_area_top_pixels: 0,
        projection_type: Some("equirectangular".to_string()),
        pose_heading_degrees: Some(123.0),
    });
    let config = TilerConfig {
        angles: partial_config,
        ..TilerConfig::default()
    };
    (img, config)
}

fn image5_config() -> (PathBuf, TilerConfig) {
    let img = Path::new("img/partial_sphere/PANO_20130622_091214.jpg").to_path_buf();
    // From exif:
    let partial_config = exif_to_partial_pano_config(&PanoExif {
        cropped_area_image_height_pixels: 807,
        cropped_area_image_width_pixels: 7896,
        full_pano_height_pixels: 3948,
        full_pano_width_pixels: 7896,
        cropped_area_top_pixels: 1604,
        projection_type: Some("equirectangular".to_string()),
        pose_heading_degrees: Some(278.0),
    });
    let config = TilerConfig {
        angles: partial_config,
        ..TilerConfig::default()
    };
    (img, config)
}

#[test]
fn test_generate_multires_panorama() {
    let img_configs = &[
        image1_config(),
        image2_config(),
        image3_config(),
        image4_config(),
        image5_config(),
    ];
    for (img, config) in img_configs {
        generate(img, config);
    }
}

fn generate(img_path: &Path, config: &TilerConfig) {
    assert!(img_path.exists());

    // Load the pano
    let dynamic_img = image::open(img_path).expect("Failed to open source integration test image");
    let rgb_img = dynamic_img.to_rgb8();

    // Process the panorama
    let pano_output = process_panorama(&rgb_img, config)
        .expect("Failed to process panorama in the tiler pipeline");

    // Save to disk
    let out_path = format!(
        "target/partial_sphere_{}",
        img_path.file_name().unwrap().to_string_lossy(),
    );
    let output_dir = Path::new(&out_path);
    if output_dir.exists() {
        std::fs::remove_dir_all(output_dir).expect("Failed to remove test output directory");
    }
    save_to_disk(
        &pano_output,
        output_dir,
        config.output.format,
        config.output.quality,
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

    println!("Output directory: {}", output_dir.display());
}
