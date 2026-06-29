#![deny(clippy::unwrap_used)]
#![allow(
    clippy::missing_errors_doc,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

//! A library to generate multi-resolution cubemap tiles and Pannellum-compatible configurations
//! from equirectangular or cylindrical panoramas.
//!
//! # Overview
//! `panorama-tiler` processes a single stitched panorama image into:
//! 1. Six cubemap face images (`front`, `back`, `up`, `down`, `left`, `right`).
//! 2. A multi-resolution pyramid containing cropped tile segments at configurable zoom levels.
//! 3. A JSON configuration file (`config.json`) mapped directly to Pannellum's configuration format.
//!
//! # Feature Flags
//! - **`metadata`** (Enabled by default): Enables automatic projection angle and crop detection using XMP and EXIF tags.
//! - **`webp`** (Enabled by default): Adds support for encoding tile output in the WebP format.
//!
//! # Examples
//!
//! ### Automatic Metadata Processing
//! If the source panorama contains valid EXIF or `GPano` XMP tags, you can process the image with
//! automatic angle extraction:
//!
//! ```rust,no_run
//! # #[cfg(feature = "metadata")]
//! # {
//! use panorama_tiler::{OutputConfig, OutputFormat, tile_panorama_with_guessed_angles};
//! use std::path::Path;
//!
//! fn main() -> Result<(), panorama_tiler::TilerError> {
//!     let input = Path::new("input_photosphere.jpg");
//!     let output = Path::new("tiles_output");
//!     
//!     let config = OutputConfig {
//!         format: OutputFormat::Webp,
//!         quality: 85,
//!         ..Default::default()
//!     };
//!
//!     tile_panorama_with_guessed_angles(input, output, Some(config))?;
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ### Manual Configuration Processing
//! If metadata tags are absent, parameters can be passed manually:
//!
//! ```rust,no_run
//! use panorama_tiler::{
//!     TilerConfig, PanoAngles, OutputConfig, Projection, OutputFormat, tile_panorama
//! };
//! use std::path::Path;
//!
//! fn main() -> Result<(), panorama_tiler::TilerError> {
//!     let config = TilerConfig {
//!         angles: PanoAngles {
//!             haov: 180.0,
//!             vaov: 90.0,
//!             projection: Projection::Cylindrical,
//!             ..Default::default()
//!         },
//!         output: OutputConfig {
//!             tile_size: 512,
//!             format: OutputFormat::Jpeg,
//!             quality: 85,
//!             ..Default::default()
//!         },
//!     };
//!
//!     tile_panorama(
//!         Path::new("input_pano.jpg"),
//!         Path::new("tiles_output"),
//!         &config,
//!     )?;
//!     Ok(())
//! }
//! ```

mod config;
mod error;
pub mod exif;
mod logic;
mod orchestrator;

pub use config::*;
pub use error::TilerError;
pub use logic::*;
pub use orchestrator::*;
