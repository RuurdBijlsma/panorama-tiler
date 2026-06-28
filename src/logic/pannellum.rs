use crate::{GeneratedTiles, PannellumConfig, TilerConfig, config};

pub fn generate_pannellum_config(
    config: &TilerConfig,
    generated_tiles: &GeneratedTiles,
    clamped_tile_size: u32,
    actual_cube_size: u32,
) -> PannellumConfig {
    let hfov = 100.0;
    let haov_opt = if config.angles.haov < 360.0 {
        Some(config.angles.haov)
    } else {
        None
    };
    let min_yaw = haov_opt.map(|h| -h / 2.0 - config.output.yaw_padding);
    let max_yaw = haov_opt.map(|h| h / 2.0 + config.output.yaw_padding);
    let yaw = haov_opt.map(|h| -h / 2.0 + hfov / 2.0);

    let vaov_opt = if config.angles.vaov < 180.0 {
        Some(config.angles.vaov)
    } else {
        None
    };
    // Section 2.B: Clamp pitch bounds within Pannellum limits [-90.0, 90.0]
    let min_pitch = vaov_opt.map(|v| {
        (-v / 2.0 + config.angles.v_offset - config.output.pitch_padding).clamp(-90.0, 90.0)
    });
    let max_pitch = vaov_opt.map(|v| {
        (v / 2.0 + config.angles.v_offset + config.output.pitch_padding).clamp(-90.0, 90.0)
    });
    let pitch = vaov_opt.map(|_| config.angles.v_offset);
    let v_offset = vaov_opt.map(|_| config.angles.v_offset);

    let background_color = if config.output.background_color != [0, 0, 0] {
        Some(
            config
                .output
                .background_color
                .iter()
                .map(|&c| c as f64 / 255.0)
                .collect(),
        )
    } else {
        None
    };

    let avoid_showing_background = if config.output.avoid_showing_background
        && (config.angles.haov < 360.0 || config.angles.vaov < 180.0)
    {
        Some(true)
    } else {
        None
    };

    let auto_load = if config.output.auto_load {
        Some(true)
    } else {
        None
    };

    let multires = config::MultiResConfig {
        sht_hash: None,
        equirectangular_thumbnail: None,
        missing_tiles: generated_tiles.missing_tiles_str.clone(),
        path: "/%l/%s%y_%x".to_string(),
        fallback_path: if config.output.fallback_size > 0 {
            Some("/fallback/%s".to_string())
        } else {
            None
        },
        extension: config.output.format.to_extension().to_owned(),
        tile_resolution: clamped_tile_size,
        max_level: generated_tiles.levels,
        cube_resolution: actual_cube_size,
    };

    PannellumConfig {
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
        north_offset: config.angles.north_offset,
        pano_type: "multires".to_string(),
        multi_res: multires,
    }
}
