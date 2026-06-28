use serde::{Deserialize, Serialize};

/// Input projection format of the source image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Projection {
    #[default]
    Equirectangular,
    Cylindrical,
}

/// Output image formats supported by the tiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Jpeg,
    Png,
    #[cfg(feature = "webp")]
    Webp,
}

impl OutputFormat {
    pub fn to_extension(&self) -> &'static str {
        match self {
            OutputFormat::Jpeg => "jpg",
            OutputFormat::Png => "png",
            #[cfg(feature = "webp")]
            OutputFormat::Webp => "webp",
        }
    }
}

/// Interpolation mode for pixel sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum InterpolationMode {
    Bilinear,
    #[default]
    Bicubic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanoAngles {
    /// Horizontal Angle of View (degrees, 0.0 to 360.0).
    pub haov: f64,
    /// Vertical Angle of View (degrees, 0.0 to 180.0).
    pub vaov: f64,
    /// Vertical pitch offset of the center (degrees).
    pub v_offset: f64,
    /// Offset of the horizon in pixels (can be negative).
    pub horizon_pixels: i32,
    pub projection: Projection,
    /// Compass heading offset of the center (degrees).
    pub north_offset: Option<f64>,
}

impl Default for PanoAngles {
    fn default() -> Self {
        Self {
            haov: 360.0,
            vaov: 180.0,
            v_offset: 0.0,
            horizon_pixels: 0,
            projection: Projection::default(),
            north_offset: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub tile_size: u32,
    pub fallback_size: u32,
    pub cube_size: u32,
    pub auto_load: bool,
    pub format: OutputFormat,
    pub quality: u8,
    pub interpolation_mode: InterpolationMode,
    pub yaw_padding: f64,
    pub pitch_padding: f64,
    /// Background color used beyond boundaries (RGB, 0-255).
    pub background_color: [u8; 3],
    /// Constrain viewport boundaries within image limits.
    pub avoid_showing_background: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            tile_size: 512,
            fallback_size: 1024,
            cube_size: 0, // 0 defaults to retaining full detail automatically
            auto_load: true,
            format: OutputFormat::default(),
            quality: 75,
            interpolation_mode: InterpolationMode::default(),
            background_color: [0, 0, 0],
            avoid_showing_background: false,
            yaw_padding: 0.0,
            pitch_padding: 0.0,
        }
    }
}

/// Global tiler options.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TilerConfig {
    pub angles: PanoAngles,
    pub output: OutputConfig,
}

// --- Serialization structures for the final output config.json ---
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PannellumConfig {
    pub hfov: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haov: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_yaw: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yaw: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_yaw: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaov: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_offset: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_pitch: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pitch: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avoid_showing_background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_load: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub north_offset: Option<f64>,
    #[serde(rename = "type")]
    pub pano_type: String, // Always "multires"
    pub multi_res: MultiResConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MultiResConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sht_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equirectangular_thumbnail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_tiles: Option<String>,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_path: Option<String>,
    pub extension: String,
    pub tile_resolution: u32,
    pub max_level: u32,
    pub cube_resolution: u32,
}
