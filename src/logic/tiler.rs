use crate::config::TilerConfig;
use crate::logic::b83;
use fast_image_resize as fr;
use image::{Rgb, RgbImage};
use rayon::prelude::*;
use std::collections::BTreeSet;

/// A representation of an individual generated tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileItem {
    pub level: u32,
    pub face: char,
    pub col: u32,
    pub row: u32,
    pub image: RgbImage,
}

/// A representation of a fallback cube face tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallbackItem {
    pub face: char,
    pub image: RgbImage,
}

/// Container holding the raw outputs of the multi-resolution pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Checks if a region in an image contains exclusively background pixels without generating allocations.
fn is_region_empty(
    img: &RgbImage,
    left: u32,
    upper: u32,
    width: u32,
    height: u32,
    bg_color: Rgb<u8>,
) -> bool {
    for y in upper..(upper + height) {
        for x in left..(left + width) {
            if *img.get_pixel(x, y) != bg_color {
                return false;
            }
        }
    }
    true
}

/// Breaks down each of the high-res faces into multi-resolution pyramids and tiles.
pub fn generate_pyramid(
    faces: &[(char, RgbImage)],
    config: &TilerConfig,
    clamped_tile_size: u32,
    actual_cube_size: u32,
) -> GeneratedTiles {
    let tile_size = clamped_tile_size.min(actual_cube_size);
    let face_letters = ['f', 'b', 'u', 'd', 'l', 'r'];
    let bg_color = Rgb(config.output.background_color);

    let levels = {
        let ratio = (actual_cube_size as f64) / (tile_size as f64);
        let mut l = (ratio.log2().ceil() as u32) + 1;
        if l >= 2 && actual_cube_size / 2u32.pow(l - 2) == tile_size {
            l -= 1; // Edge case matching Python script adjustments
        }
        l
    };

    let mut level_sizes = vec![0; (levels + 1) as usize];
    let mut current_size = actual_cube_size;
    for level in (1..=levels).rev() {
        level_sizes[level as usize] = current_size;
        current_size /= 2;
    }

    let is_partial = config.angles.haov < 360.0 || config.angles.vaov < 180.0;

    // Generate pyramids across all faces in parallel
    let (tiles_nested, missing_nested): (Vec<Vec<TileItem>>, Vec<Vec<MissingTile>>) = faces
        .par_iter()
        .enumerate()
        .map(|(f_idx, &(letter, ref full_face))| {
            let mut local_tiles = Vec::new();
            let mut local_missing = Vec::new();
            let mut current_face = full_face.clone();

            let mut resizer = fr::Resizer::new();
            let resize_options = fr::ResizeOptions::new()
                .resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));

            for level in (1..=levels).rev() {
                let size = level_sizes[level as usize];
                let num_tiles_wide_high = ((size as f64) / (tile_size as f64)).ceil() as u32;

                if level < levels {
                    // Downscale face recursively using Lanczos3
                    let mut downscaled = RgbImage::new(size, size);
                    resizer
                        .resize(&current_face, &mut downscaled, Some(&resize_options))
                        .expect("Failed to downscale cube face level");
                    current_face = downscaled;
                }

                for row in 0..num_tiles_wide_high {
                    for col in 0..num_tiles_wide_high {
                        let left = col * tile_size;
                        let upper = row * tile_size;
                        let width = tile_size.min(size - left);
                        let height = tile_size.min(size - upper);

                        // Avoid allocating new sub-images if the entire region is verified empty
                        if is_partial
                            && is_region_empty(&current_face, left, upper, width, height, bg_color)
                        {
                            local_missing.push(MissingTile {
                                face_idx: f_idx,
                                level,
                                col,
                                row,
                            });
                        } else {
                            let tile_crop = image::imageops::crop_imm(
                                &current_face,
                                left,
                                upper,
                                width,
                                height,
                            )
                            .to_image();
                            local_tiles.push(TileItem {
                                level,
                                face: letter,
                                col,
                                row,
                                image: tile_crop,
                            });
                        }
                    }
                }
            }
            (local_tiles, local_missing)
        })
        .unzip();

    let tiles: Vec<TileItem> = tiles_nested.into_iter().flatten().collect();
    let missing_tiles: Vec<MissingTile> = missing_nested.into_iter().flatten().collect();

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
                missing_str.push_str(&b83::encode(&[mt.level], 1));
                let level_size = level_sizes[mt.level as usize];
                let max_tile_num = ((level_size as f64) / (tile_size as f64)).ceil() as u32 - 1;
                num_tile_digits = (((max_tile_num + 1) as f64).log(83.0).ceil() as usize).max(1);
            }
            missing_str.push_str(&b83::encode(&[mt.col, mt.row], num_tile_digits));
            prev_face = Some(mt.face_idx);
            prev_level = Some(mt.level);
        }
        Some(missing_str)
    } else {
        None
    };

    // Generate fallback files if fallback size is defined
    let mut fallback_tiles = Vec::new();
    if config.output.fallback_size > 0 {
        let mut resizer = fr::Resizer::new();
        let resize_options = fr::ResizeOptions::new()
            .resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));

        for &(letter, ref full_face) in faces {
            let mut resized =
                RgbImage::new(config.output.fallback_size, config.output.fallback_size);
            resizer
                .resize(full_face, &mut resized, Some(&resize_options))
                .expect("Failed to resize fallback face");

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
