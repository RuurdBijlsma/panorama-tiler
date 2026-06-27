use image::{RgbImage, Rgb};
use std::collections::BTreeSet;
use crate::config::TilerConfig;

/// A representation of an individual generated tile.
pub struct TileItem {
    pub level: u32,
    pub face: char,
    pub col: u32,
    pub row: u32,
    pub image: RgbImage,
}

/// A representation of a fallback cube face tile.
pub struct FallbackItem {
    pub face: char,
    pub image: RgbImage,
}

/// Container holding the raw outputs of the multi-resolution pipeline.
pub struct GeneratedTiles {
    pub tiles: Vec<TileItem>,
    pub fallback_tiles: Vec<FallbackItem>,
    pub missing_tiles_str: Option<String>,
    pub levels: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MissingTile {
    face_idx: usize,
    level: u32,
    col: u32,
    row: u32,
}

/// Breaks down each of the high-res faces into multi-resolution pyramids and tiles.
pub fn generate_pyramid(
    faces: &[(char, RgbImage)],
    config: &TilerConfig,
    actual_cube_size: u32,
) -> GeneratedTiles {
    let tile_size = config.tile_size;
    let face_letters = ['f', 'b', 'u', 'd', 'l', 'r'];

    // Map background color to u8 values
    let bg_color = Rgb([
        (config.partial_config.background_color[0] * 255.0).round().clamp(0.0, 255.0) as u8,
        (config.partial_config.background_color[1] * 255.0).round().clamp(0.0, 255.0) as u8,
        (config.partial_config.background_color[2] * 255.0).round().clamp(0.0, 255.0) as u8,
    ]);

    // Calculate maximum pyramid levels
    let levels = {
        let ratio = (actual_cube_size as f64) / (tile_size as f64);
        let mut l = (ratio.log2().ceil() as u32) + 1;
        if l >= 2 && actual_cube_size / 2u32.pow(l - 2) == tile_size {
            l -= 1; // Edge case matching Python script adjustments
        }
        l
    };

    let mut tiles = Vec::new();
    let mut missing_tiles = Vec::new();

    for (f_idx, &(letter, ref full_face)) in faces.iter().enumerate() {
        let mut size = actual_cube_size;
        let mut current_face = full_face.clone();

        for level in (1..=levels).rev() {
            let num_tiles_wide_high = ((size as f64) / (tile_size as f64)).ceil() as u32;

            if level < levels {
                // Downscale face recursively using Lanczos algorithm
                current_face = image::imageops::resize(
                    &current_face,
                    size,
                    size,
                    image::imageops::FilterType::Lanczos3,
                );
            }

            for row in 0..num_tiles_wide_high {
                for col in 0..num_tiles_wide_high {
                    let left = col * tile_size;
                    let upper = row * tile_size;
                    let width = tile_size.min(size - left);
                    let height = tile_size.min(size - upper);

                    let tile_crop = image::imageops::crop_imm(&current_face, left, upper, width, height).to_image();

                    // Check if the cropped tile contains exclusively background pixels
                    let is_empty = tile_crop.pixels().all(|&pixel| pixel == bg_color);

                    // For partial panoramas, discard blank background tiles
                    let is_partial = config.partial_config.haov < 360.0 || config.partial_config.vaov < 180.0;
                    if is_partial && is_empty {
                        missing_tiles.push(MissingTile {
                            face_idx: f_idx,
                            level,
                            col,
                            row,
                        });
                    } else {
                        tiles.push(TileItem {
                            level,
                            face: letter,
                            col,
                            row,
                            image: tile_crop,
                        });
                    }
                }
            }
            size /= 2;
        }
    }

    // Process missing tiles string
    let missing_tiles_str = if !missing_tiles.is_empty() {
        let mut missing_set: BTreeSet<MissingTile> = missing_tiles.into_iter().collect();

        // Strip children of missing parents to save space
        let mut redundant = Vec::new();
        for &t in &missing_set {
            if t.level > 1 {
                let parent = MissingTile {
                    face_idx: t.face_idx,
                    level: t.level - 1,
                    col: t.col / 2,
                    row: t.row / 2,
                };
                if missing_set.contains(&parent) {
                    redundant.push(t);
                }
            }
        }
        for r in redundant {
            missing_set.remove(&r);
        }

        // Format and Base83 compress the remaining missing tile configurations
        let mut sorted_missing: Vec<MissingTile> = missing_set.into_iter().collect();
        sorted_missing.sort();

        let mut missing_str = String::new();
        let mut prev_face: Option<usize> = None;
        let mut prev_level: Option<u32> = None;
        let mut num_tile_digits = 1;

        for mt in sorted_missing {
            if Some(mt.face_idx) != prev_face {
                missing_str.push('!');
                missing_str.push(face_letters[mt.face_idx]);
            }
            if Some(mt.level) != prev_level {
                missing_str.push('>');
                missing_str.push_str(&crate::b83::encode(&[mt.level], 1));

                let level_size = actual_cube_size / 2u32.pow(levels - mt.level);
                let max_tile_num = ((level_size as f64) / (tile_size as f64)).ceil() as u32 - 1;
                num_tile_digits = (((max_tile_num + 1) as f64).log(83.0).ceil() as usize).max(1);
            }
            missing_str.push_str(&crate::b83::encode(&[mt.col, mt.row], num_tile_digits));
            prev_face = Some(mt.face_idx);
            prev_level = Some(mt.level);
        }
        Some(missing_str)
    } else {
        None
    };

    // Generate fallback files if fallback size is defined
    let mut fallback_tiles = Vec::new();
    if config.fallback_size > 0 {
        for &(letter, ref full_face) in faces {
            let resized = image::imageops::resize(
                full_face,
                config.fallback_size,
                config.fallback_size,
                image::imageops::FilterType::Lanczos3,
            );
            fallback_tiles.push(FallbackItem {
                face: letter,
                image: resized,
            });
        }
    }

    GeneratedTiles {
        tiles,
        fallback_tiles,
        missing_tiles_str,
        levels,
    }
}