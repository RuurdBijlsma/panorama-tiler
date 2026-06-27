use crate::TilerConfig;
use image::Rgb;

pub fn get_bg_color(config: &TilerConfig) -> Rgb<u8> {
    // Map background color to u8 values
    Rgb([
        (config.partial_config.background_color[0] * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8,
        (config.partial_config.background_color[1] * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8,
        (config.partial_config.background_color[2] * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8,
    ])
}
