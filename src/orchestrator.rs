use crate::config::OutputConfig;
use crate::exif::guess_pano_angles_from_bytes;
use crate::{
    GeneratedTiles, OutputFormat, PannellumConfig, TilerConfig, TilerError, generate_cube_faces,
    generate_pannellum_config, generate_pyramid, save_image,
};
use image::RgbImage;
use rayon::prelude::*;
use std::fs;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct TiledPanoramaOutput {
    pub generated_tiles: GeneratedTiles,
    pub pannellum_config: PannellumConfig,
    pub actual_cube_size: u32,
}

/// Process an RgbImage into Pannellum tiles and config.
pub fn process_panorama(
    src_image: &RgbImage,
    config: &TilerConfig,
) -> Result<TiledPanoramaOutput, TilerError> {
    let (width, _) = src_image.dimensions();

    if !config.angles.haov.is_finite()
        || !config.angles.vaov.is_finite()
        || config.angles.haov <= 0.0
        || config.angles.vaov <= 0.0
    {
        return Err(TilerError::InvalidConfig(
            "Both `haov` and `vaov` must be positive, finite numbers".to_owned(),
        ));
    }

    // Calculate target cube resolution based on input horizontal field of view
    let actual_cube_size = if config.output.cube_size != 0 {
        config.output.cube_size
    } else {
        let ratio = 360.0 / config.angles.haov;
        let computed = ratio * (width as f64) / std::f64::consts::PI;
        8 * ((computed / 8.0) as u32)
    };
    let clamped_tile_size = config.output.tile_size.min(actual_cube_size);

    // Pipeline: generate cube faces, pyramid and pannellum config
    let faces = generate_cube_faces(src_image, config, actual_cube_size);
    let generated_tiles = generate_pyramid(&faces, config, clamped_tile_size, actual_cube_size);
    let p_config = generate_pannellum_config(
        config,
        &generated_tiles,
        clamped_tile_size,
        actual_cube_size,
    );

    Ok(TiledPanoramaOutput {
        pannellum_config: p_config,
        generated_tiles,
        actual_cube_size,
    })
}

/// Save generated tiles and the Pannellum config.json file to a given directory.
pub fn save_to_disk(
    pano: &TiledPanoramaOutput,
    output_dir: &Path,
    output_format: OutputFormat,
    quality: u8,
) -> Result<(), TilerError> {
    fs::create_dir_all(output_dir)?;

    let ext = output_format.to_extension();

    // Create zoom level folders
    for level in 1..=pano.generated_tiles.levels {
        let level_dir = output_dir.join(level.to_string());
        fs::create_dir_all(&level_dir)?;
    }

    pano.generated_tiles
        .tiles
        .par_iter()
        .try_for_each(|tile| -> Result<(), TilerError> {
            let filename = format!("{}{}_{}.{}", tile.face, tile.row, tile.col, ext);
            let filepath = output_dir.join(tile.level.to_string()).join(filename);
            save_image(&tile.image, &filepath, output_format, quality)?;
            Ok(())
        })?;

    if !pano.generated_tiles.fallback_tiles.is_empty() {
        let fallback_dir = output_dir.join("fallback");
        fs::create_dir_all(&fallback_dir)?;

        pano.generated_tiles
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
    serde_json::to_writer_pretty(file, &pano.pannellum_config)?;

    Ok(())
}

/// Load a pano, auto-detect its angles via EXIF and XMP metadata, generate the multi-resolution
/// tiles, and write the output to a directory.
///
/// If no metadata is found, it will fall back to aspect-ratio-based heuristics.
pub fn tile_panorama_with_guessed_angles(
    input_file: &Path,
    output_dir: &Path,
    output_config: Option<OutputConfig>,
) -> Result<(), TilerError> {
    let bytes = fs::read(input_file)?;
    let config = TilerConfig {
        angles: guess_pano_angles_from_bytes(&bytes)?,
        output: output_config.unwrap_or_default(),
    };

    let dynamic_img = image::load_from_memory(&bytes)?;
    let rgb_img = dynamic_img.to_rgb8();
    let pano_output = process_panorama(&rgb_img, &config)?;

    save_to_disk(
        &pano_output,
        output_dir,
        config.output.format,
        config.output.quality,
    )?;

    Ok(())
}

/// Load a pano, generate the multi-resolution tiles, and write the output to a directory.
pub fn tile_panorama(
    input_file: &Path,
    output_dir: &Path,
    config: &TilerConfig,
) -> Result<(), TilerError> {
    let bytes = fs::read(input_file)?;
    let dynamic_img = image::load_from_memory(&bytes)?;
    let rgb_img = dynamic_img.to_rgb8();
    let pano_output = process_panorama(&rgb_img, config)?;

    save_to_disk(
        &pano_output,
        output_dir,
        config.output.format,
        config.output.quality,
    )?;

    Ok(())
}
