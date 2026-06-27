use serde::{Serialize, Deserialize};

/// Input projection format of the source image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Projection {
    Equirectangular,
    Cylindrical,
}

/// Parameters for partial panorama mapping configurations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialPanoConfig {
    /// Horizontal Angle of View (degrees, 0.0 to 360.0).
    pub haov: f64,
    /// Vertical Angle of View (degrees, 0.0 to 180.0).
    pub vaov: f64,
    /// Vertical pitch offset of the center (degrees).
    pub v_offset: f64,
    /// Offset of the horizon in pixels (can be negative).
    pub horizon_pixels: i32,
    /// Background color used beyond boundaries (RGB, normalized 0.0 to 1.0).
    pub background_color: [f64; 3],
    /// Constrain viewport boundaries within image limits.
    pub avoid_showing_background: bool,
}

impl Default for PartialPanoConfig {
    fn default() -> Self {
        Self {
            haov: 360.0,
            vaov: 180.0,
            v_offset: 0.0,
            horizon_pixels: 0,
            background_color: [0.0, 0.0, 0.0],
            avoid_showing_background: false,
        }
    }
}

/// Global tiler options.
#[derive(Debug, Clone)]
pub struct TilerConfig {
    pub projection: Projection,
    pub partial_config: PartialPanoConfig,
    pub tile_size: u32,
    pub fallback_size: u32,
    pub cube_size: u32,
    pub auto_load: bool,
    pub png_output: bool,
    pub quality: u8,
}

impl Default for TilerConfig {
    fn default() -> Self {
        Self {
            projection: Projection::Equirectangular,
            partial_config: PartialPanoConfig::default(),
            tile_size: 512,
            fallback_size: 1024,
            cube_size: 0, // 0 defaults to retaining full detail automatically
            auto_load: false,
            png_output: false,
            quality: 75,
        }
    }
}

// --- Serialization structures for the final output config.json ---
#[derive(Serialize, Deserialize, Debug, Clone)]
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
    #[serde(rename = "type")]
    pub pano_type: String, // Always "multires"
    pub multi_res: MultiResConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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