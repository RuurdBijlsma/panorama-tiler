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
    #[must_use]
    pub const fn to_extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            #[cfg(feature = "webp")]
            Self::Webp => "webp",
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

/// Downscaling method used for lower-resolution pyramid levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DownscalingMethod {
    /// Recursively downscale from the previous level. Faster but can accumulate interpolation errors.
    #[default]
    Recursive,
    /// Downscale directly from the full-resolution cube face for each level. Slower but preserves maximum sharpness.
    Direct,
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
    pub format: OutputFormat,
    pub quality: u8,
    pub interpolation_mode: InterpolationMode,
    pub yaw_padding: f64,
    pub pitch_padding: f64,
    /// Background color used beyond boundaries (RGB, 0-255).
    pub background_color: [u8; 3],
    /// Method used to downscale cube faces to generate lower resolution pyramid levels.
    pub downscaling_method: DownscalingMethod,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            tile_size: 512,
            fallback_size: 1024,
            cube_size: 0, // 0 defaults to retaining full detail automatically
            format: OutputFormat::default(),
            quality: 75,
            interpolation_mode: InterpolationMode::default(),
            background_color: [0, 0, 0],
            yaw_padding: 0.0,
            pitch_padding: 0.0,
            downscaling_method: DownscalingMethod::default(),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haov: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_yaw: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_yaw: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaov: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_offset: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_pitch: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pitch: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub north_offset: Option<f64>,
    #[serde(rename = "type")]
    pub pano_type: String, // Always "multires"
    pub multi_res: MultiResConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MultiResConfig {
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
