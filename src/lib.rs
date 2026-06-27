pub mod b83;
pub mod config;
pub mod error;
pub mod projection;
pub mod tiler;

use std::path::Path;
use std::fs::{self, File};
use std::io::BufWriter;
use image::{RgbImage, codecs::jpeg::JpegEncoder};

pub use config::{TilerConfig, PartialPanoConfig, Projection, PannellumConfig};
pub use error::{Result, TilerError};
pub use tiler::GeneratedTiles;

// todo refactor logic into separate file

/// High-level orchestration function to process an RgbImage into Pannellum tiles and config.
pub fn process_panorama(
    src_image: &RgbImage,
    config: &TilerConfig,
) -> Result<(GeneratedTiles, PannellumConfig, u32)> {
    let (width, height) = src_image.dimensions();

    // Auto-detect HAOV / VAOV if left at -1.0 default matching Python script
    let mut haov = config.partial_config.haov;
    if haov == -1.0 {
        if config.projection == Projection::Cylindrical || (width as f64) / (height as f64) == 2.0 {
            haov = 360.0;
        } else {
            return Err(TilerError::InvalidConfig(
                "Unless given the --haov option, equirectangular input image must be a full (not partial) panorama!".to_string()
            ));
        }
    }

    let mut vaov = config.partial_config.vaov;
    if vaov == -1.0 {
        if config.projection == Projection::Cylindrical || (width as f64) / (height as f64) == 2.0 {
            vaov = 180.0;
        } else {
            return Err(TilerError::InvalidConfig(
                "Unless given the --vaov option, equirectangular input image must be a full (not partial) panorama!".to_string()
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

    // 1. Generate high-resolution cube faces
    let faces = projection::generate_cube_faces(src_image, &resolved_config, actual_cube_size);

    // 2. Generate tiles pyramid structure
    let generated_tiles = tiler::generate_pyramid(&faces, &resolved_config, actual_cube_size);

    // 3. Build serialization configuration structure for Pannellum
    let hfov = 100.0; // Standard default field of view
    let haov_opt = if haov < 360.0 { Some(haov) } else { None };
    let min_yaw = haov_opt.map(|h| -h / 2.0);
    let max_yaw = haov_opt.map(|h| h / 2.0);
    let yaw = haov_opt.map(|h| -h / 2.0 + hfov / 2.0);

    let vaov_opt = if vaov < 180.0 { Some(vaov) } else { None };
    let min_pitch = vaov_opt.map(|v| -v / 2.0 + config.partial_config.v_offset);
    let max_pitch = vaov_opt.map(|v| v / 2.0 + config.partial_config.v_offset);
    let pitch = vaov_opt.map(|_| config.partial_config.v_offset);
    let v_offset = vaov_opt.map(|_| config.partial_config.v_offset);

    let background_color = if config.partial_config.background_color != [0.0, 0.0, 0.0] {
        Some(config.partial_config.background_color.to_vec())
    } else {
        None
    };

    let avoid_showing_background = if config.partial_config.avoid_showing_background && (haov < 360.0 || vaov < 180.0) {
        Some(true)
    } else {
        None
    };

    let auto_load = if config.auto_load { Some(true) } else { None };
    let extension = if config.png_output { "png".to_string() } else { "jpg".to_string() };

    let multires = config::MultiResConfig {
        sht_hash: None, // Excluded for simplicity (requires heavy pyshtools library replacement)
        equirectangular_thumbnail: None,
        missing_tiles: generated_tiles.missing_tiles_str.clone(),
        path: format!("/%l/%s%y_%x.{}", extension),
        fallback_path: if config.fallback_size > 0 { Some("/fallback/%s".to_string()) } else { None },
        extension,
        tile_resolution: config.tile_size,
        max_level: generated_tiles.levels,
        cube_resolution: actual_cube_size,
    };

    let p_config = config::PannellumConfig {
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
        pano_type: "multires".to_string(),
        multi_res: multires,
    };

    Ok((generated_tiles, p_config, actual_cube_size))
}

/// Helper function to save generated tiles and the config.json file to a given directory.
pub fn save_to_disk(
    generated: &GeneratedTiles,
    config_json: &config::PannellumConfig,
    output_dir: &Path,
    png_output: bool,
    quality: u8,
) -> Result<()> {
    fs::create_dir_all(output_dir)?;

    // 1. Save standard multires tiles
    for tile in &generated.tiles {
        let level_dir = output_dir.join(tile.level.to_string());
        fs::create_dir_all(&level_dir)?;

        let ext = if png_output { "png" } else { "jpg" };
        let filename = format!("{}{}_{}.{}", tile.face, tile.row, tile.col, ext);
        let filepath = level_dir.join(filename);

        if png_output {
            tile.image.save(&filepath)?;
        } else {
            let file = File::create(&filepath)?;
            let ref mut writer = BufWriter::new(file);
            let mut encoder = JpegEncoder::new_with_quality(writer, quality);
            encoder.encode_image(&tile.image)?;
        }
    }

    // 2. Save fallback tiles
    if !generated.fallback_tiles.is_empty() {
        let fallback_dir = output_dir.join("fallback");
        fs::create_dir_all(&fallback_dir)?;

        let ext = if png_output { "png" } else { "jpg" };
        for fallback in &generated.fallback_tiles {
            let filename = format!("{}.{}", fallback.face, ext);
            let filepath = fallback_dir.join(filename);

            if png_output {
                fallback.image.save(&filepath)?;
            } else {
                let file = File::create(&filepath)?;
                let ref mut writer = BufWriter::new(file);
                let mut encoder = JpegEncoder::new_with_quality(writer, quality);
                encoder.encode_image(&fallback.image)?;
            }
        }
    }

    // 3. Save config.json
    let config_path = output_dir.join("config.json");
    let file = File::create(config_path)?;
    serde_json::to_writer_pretty(file, config_json)?;

    Ok(())
}