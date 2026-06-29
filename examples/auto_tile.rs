use panorama_tiler::{OutputConfig, OutputFormat, tile_panorama_with_guessed_angles};
use std::path::Path;

fn main() -> Result<(), panorama_tiler::TilerError> {
    let input_path = Path::new("img/sphere/PXL_20220918_115954889.PHOTOSPHERE.jpg");
    let output_dir = Path::new("tiles_output");

    let output_config = OutputConfig {
        format: OutputFormat::Webp,
        quality: 85,
        ..Default::default()
    };

    tile_panorama_with_guessed_angles(
        input_path,
        output_dir,
        Some(output_config),
    )?;

    Ok(())
}