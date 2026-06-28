use crate::{GeneratedTiles, GeneratorConfig, OutputFormat, PannellumConfig, Projection, TilerError, config, projection, tiler, PanoExif, exif_to_partial_pano_config, PartialPanoConfig};
use image::RgbImage;
use image::codecs::jpeg::JpegEncoder;
use rayon::prelude::*;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// High-level orchestration function to process an RgbImage into Pannellum tiles and config.
pub fn process_panorama(
    src_image: &RgbImage,
    config: &GeneratorConfig,
) -> Result<(GeneratedTiles, PannellumConfig, u32), TilerError> {
    let (width, height) = src_image.dimensions();

    // Auto-detect HAOV / VAOV if left at -1.0 default
    let mut haov = config.partial_config.haov;
    if haov == -1.0 {
        if config.projection == Projection::Cylindrical || (width as f64) / (height as f64) == 2.0 {
            haov = 360.0;
        } else {
            return Err(TilerError::InvalidConfig(
                "Unless given a `haov` config, equirectangular input image must be a full (not partial) panorama!".to_string()
            ));
        }
    }

    let mut vaov = config.partial_config.vaov;
    if vaov == -1.0 {
        if config.projection == Projection::Cylindrical || (width as f64) / (height as f64) == 2.0 {
            vaov = 180.0;
        } else {
            return Err(TilerError::InvalidConfig(
                "Unless given the `vaov` option, equirectangular input image must be a full (not partial) panorama!".to_string()
            ));
        }
    }

    // Override configuration parameters with resolved values
    let mut resolved_config = config.clone();
    resolved_config.partial_config.haov = haov;
    resolved_config.partial_config.vaov = vaov;

    // Calculate target cube resolution based on input horizontal field of view
    let actual_cube_size = if resolved_config.cube_size != 0 {
        resolved_config.cube_size
    } else {
        let ratio = 360.0 / haov;
        let computed = ratio * (width as f64) / std::f64::consts::PI;
        8 * ((computed / 8.0) as u32)
    };

    let clamped_tile_size = resolved_config.tile_size.min(actual_cube_size);
    resolved_config.tile_size = clamped_tile_size;

    // 1. Generate high-resolution cube faces
    let faces = projection::generate_cube_faces(src_image, &resolved_config, actual_cube_size);

    // 2. Generate tiles pyramid structure
    let generated_tiles = tiler::generate_pyramid(&faces, &resolved_config, actual_cube_size);

    // 3. Build serialization configuration structure for Pannellum
    let hfov = 100.0;
    let haov_opt = if haov < 360.0 { Some(haov) } else { None };
    let min_yaw = haov_opt.map(|h| -h / 2.0 - config.yaw_padding);
    let max_yaw = haov_opt.map(|h| h / 2.0 + config.yaw_padding);
    let yaw = haov_opt.map(|h| -h / 2.0 + hfov / 2.0);

    let vaov_opt = if vaov < 180.0 { Some(vaov) } else { None };
    let min_pitch =
        vaov_opt.map(|v| -v / 2.0 + config.partial_config.v_offset - config.pitch_padding);
    let max_pitch =
        vaov_opt.map(|v| v / 2.0 + config.partial_config.v_offset + config.pitch_padding);
    let pitch = vaov_opt.map(|_| config.partial_config.v_offset);
    let v_offset = vaov_opt.map(|_| config.partial_config.v_offset);

    let background_color = if config.background_color != [0.0, 0.0, 0.0] {
        Some(config.background_color.to_vec())
    } else {
        None
    };

    let avoid_showing_background =
        if config.avoid_showing_background && (haov < 360.0 || vaov < 180.0) {
            Some(true)
        } else {
            None
        };

    let auto_load = if config.auto_load { Some(true) } else { None };

    let multires = config::MultiResConfig {
        sht_hash: None,
        equirectangular_thumbnail: None,
        missing_tiles: generated_tiles.missing_tiles_str.clone(),
        path: "/%l/%s%y_%x".to_string(),
        fallback_path: if config.fallback_size > 0 {
            Some("/fallback/%s".to_string())
        } else {
            None
        },
        extension: config.output_format.to_extension().to_owned(),
        tile_resolution: resolved_config.tile_size,
        max_level: generated_tiles.levels,
        cube_resolution: actual_cube_size,
    };

    let p_config = PannellumConfig {
        hfov,
        haov: haov_opt,
        min_yaw,
        yaw,
        max_yaw,
        vaov: vaov_opt,
        v_offset,
        min_pitch,
        pitch,
        max_pitch,
        background_color,
        avoid_showing_background,
        auto_load,
        north_offset: config.north_offset,
        pano_type: "multires".to_string(),
        multi_res: multires,
    };

    Ok((generated_tiles, p_config, actual_cube_size))
}

/// Helper function to save generated tiles and the config.json file to a given directory.
pub fn save_to_disk(
    generated: &GeneratedTiles,
    config_json: &PannellumConfig,
    output_dir: &Path,
    output_format: OutputFormat,
    quality: u8,
) -> Result<(), TilerError> {
    fs::create_dir_all(output_dir)?;

    let ext = output_format.to_extension();

    // Create zoom level folders
    for level in 1..=generated.levels {
        let level_dir = output_dir.join(level.to_string());
        fs::create_dir_all(&level_dir)?;
    }

    generated
        .tiles
        .par_iter()
        .try_for_each(|tile| -> Result<(), TilerError> {
            let filename = format!("{}{}_{}.{}", tile.face, tile.row, tile.col, ext);
            let filepath = output_dir.join(tile.level.to_string()).join(filename);
            save_image(&tile.image, &filepath, output_format, quality)?;
            Ok(())
        })?;

    if !generated.fallback_tiles.is_empty() {
        let fallback_dir = output_dir.join("fallback");
        fs::create_dir_all(&fallback_dir)?;

        generated
            .fallback_tiles
            .par_iter()
            .try_for_each(|fallback| -> Result<(), TilerError> {
                let filename = format!("{}.{}", fallback.face, ext);
                let filepath = fallback_dir.join(filename);
                save_image(&fallback.image, &filepath, output_format, quality)?;
                Ok(())
            })?;
    }

    let config_path = output_dir.join("config.json");
    let file = File::create(config_path)?;
    serde_json::to_writer_pretty(file, config_json)?;

    Ok(())
}

fn save_image(
    image: &RgbImage,
    filepath: &Path,
    output_format: OutputFormat,
    quality: u8,
) -> Result<(), TilerError> {
    match output_format {
        OutputFormat::Png => {
            image.save(filepath)?;
        }
        OutputFormat::Jpeg => {
            let file = File::create(filepath)?;
            let mut writer = BufWriter::new(file);
            let mut encoder = JpegEncoder::new_with_quality(&mut writer, quality);
            encoder.encode_image(image)?;
        }
        #[cfg(feature = "webp")]
        OutputFormat::Webp => {
            let (width, height) = image.dimensions();
            let encoder = webp::Encoder::from_rgb(image.as_raw(), width, height);
            let encoded_webp = encoder.encode(quality as f32);
            fs::write(filepath, &*encoded_webp)?;
        }
    }
    Ok(())
}

/// High-level function to load an image, auto-detect its geometry and pose via
/// EXIF and GPano XMP metadata, generate the multi-resolution tiles, and write
/// the output to a directory.
///
/// If no metadata is found, it will fallback to aspect-ratio-based heuristics.
pub fn tile_panorama_file(
    input_file: &Path,
    output_dir: &Path,
    custom_config: Option<GeneratorConfig>,
) -> Result<(), TilerError> {
    // 1. Load the actual image file to guarantee exact input resolution
    let dynamic_img = image::open(input_file)?;
    let rgb_img = dynamic_img.to_rgb8();
    let (width, height) = rgb_img.dimensions();

    // 2. Setup initial configuration
    let mut config = custom_config.unwrap_or_default();

    // 3. Extract EXIF attributes
    let mut exif_metadata = None;
    if let Ok(file) = File::open(input_file) {
        let mut buf_reader = std::io::BufReader::new(file);
        let exif_reader = exif::Reader::new();
        if let Ok(parsed_exif) = exif_reader.read_from_container(&mut buf_reader) {
            exif_metadata = Some(parsed_exif);
        }
    }

    // 4. Extract XMP metadata packets
    let mut xmp_metadata = None;
    let mut xmp_file = xmpkit::XmpFile::new();
    if xmp_file.open(input_file).is_ok() {
        if let Some(parsed_xmp) = xmp_file.get_xmp() {
            xmp_metadata = Some(parsed_xmp.clone());
        }
    }

    // Initialize detection variables
    let mut detected_gpano = false;
    let mut haov = None;
    let mut vaov = None;
    let mut v_offset = None;
    let mut horizon_pixels = None;
    let mut north_offset = None;
    let mut projection = Projection::Equirectangular;

    // 5. Query Google Photo Sphere (GPano) values
    if let Some(meta) = &xmp_metadata {
        let gpano_ns = "http://ns.google.com/photos/1.0/panorama/";

        // Helper to query and safely convert arbitrary XmpValue stringified forms
        let get_gpano_f64 = |name: &str| -> Option<f64> {
            meta.get_property(gpano_ns, name)
                .and_then(|v| v.to_string().parse::<f64>().ok())
        };

        let projection_type = meta.get_property(gpano_ns, "ProjectionType")
            .map(|v| v.to_string());

        let cropped_area_height = get_gpano_f64("CroppedAreaImageHeightPixels");
        let cropped_area_width = get_gpano_f64("CroppedAreaImageWidthPixels");
        let full_pano_height = get_gpano_f64("FullPanoHeightPixels");
        let full_pano_width = get_gpano_f64("FullPanoWidthPixels");
        let cropped_area_top = get_gpano_f64("CroppedAreaTopPixels");
        let pose_heading = get_gpano_f64("PoseHeadingDegrees");

        if let Some(ref proj_type) = projection_type {
            if proj_type == "cylindrical" {
                projection = Projection::Cylindrical;
            } else {
                projection = Projection::Equirectangular;
            }
        }

        // Check if we have complete partial photo sphere crop boundaries
        if let (Some(cropped_w), Some(cropped_h), Some(full_w), Some(full_h), Some(cropped_t)) =
            (cropped_area_width, cropped_area_height, full_pano_width, full_pano_height, cropped_area_top)
        {
            detected_gpano = true;

            // Build temporary EXIF container to derive consistent angles and offsets
            let exif_info = PanoExif {
                full_pano_width_pixels: full_w as u32,
                full_pano_height_pixels: full_h as u32,
                cropped_area_top_pixels: cropped_t as u32,
                cropped_area_image_width_pixels: cropped_w as u32,
                cropped_area_image_height_pixels: cropped_h as u32,
            };

            let partial_config = exif_to_partial_pano_config(&exif_info);
            haov = Some(partial_config.haov);
            vaov = Some(partial_config.vaov);
            v_offset = Some(partial_config.v_offset);
            horizon_pixels = Some(partial_config.horizon_pixels);
            north_offset = pose_heading;
        } else if let Some(pose_heading) = pose_heading {
            north_offset = Some(pose_heading);
        }
    }

    // 6. Cylindrical Sweep detection via focal length EXIF tags
    if !detected_gpano {
        let mut focal_length_35mm = None;

        if let Some(exif) = &exif_metadata {
            if let Some(field) = exif.get_field(exif::Tag::FocalLengthIn35mmFilm, exif::In::PRIMARY) {
                focal_length_35mm = match &field.value {
                    exif::Value::Rational(rationals) => rationals.first().map(|r| r.to_f64()),
                    _ => field.value.get_uint(0).map(|v| v as f64),
                };
            }
        }

        if let Some(focal) = focal_length_35mm {
            if focal > 0.0 {
                projection = Projection::Cylindrical;
                let crop_factor = 0.90; // Standard crop loss ratio for panorama stitches
                if let Some(angles) = crate::helpers::config_helper::calculate_pano_angles(
                    focal,
                    width,
                    height,
                    crop_factor,
                ) {
                    haov = Some(angles.haov);
                    vaov = Some(angles.vaov);
                }
            }
        }
    }

    // 7. Fallback heuristics for un-tagged panoramic images
    let aspect_ratio = width as f64 / height as f64;
    if haov.is_none() && vaov.is_none() {
        if (aspect_ratio - 2.0).abs() <= 0.1 {
            // Equirectangular Full Photo Sphere
            projection = Projection::Equirectangular;
            haov = Some(360.0);
            vaov = Some(180.0);
        } else if aspect_ratio >= 2.2 {
            // Wide image treated as a custom cylindrical sweep (e.g. 24mm default sweep equivalent)
            projection = Projection::Cylindrical;
            let crop_factor = 0.90;
            if let Some(angles) = crate::helpers::config_helper::calculate_pano_angles(
                24.0,
                width,
                height,
                crop_factor,
            ) {
                haov = Some(angles.haov);
                vaov = Some(angles.vaov);
            }
        } else {
            // Default equirectangular
            projection = Projection::Equirectangular;
            haov = Some(360.0);
            vaov = Some(180.0);
        }
    }

    // 8. Apply calculated adjustments to local generator config
    config.projection = projection;
    config.partial_config = PartialPanoConfig {
        haov: haov.unwrap_or(360.0),
        vaov: vaov.unwrap_or(180.0),
        v_offset: v_offset.unwrap_or(0.0),
        horizon_pixels: horizon_pixels.unwrap_or(0),
    };
    if north_offset.is_some() {
        config.north_offset = north_offset;
    }

    // 9. Execute high-performance multi-resolution processing pipeline
    let (tiles, config_json, _) = process_panorama(&rgb_img, &config)?;

    // 10. Write final multi-res hierarchy files to target path
    save_to_disk(
        &tiles,
        &config_json,
        output_dir,
        config.output_format,
        config.quality,
    )?;

    Ok(())
}
