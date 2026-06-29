use crate::config::{InterpolationMode, Projection, TilerConfig};
use image::{Rgb, RgbImage};
use rayon::prelude::*;
use std::f64::consts::{FRAC_PI_2, PI};

/// Generates the 6 cubemap face images from an equirectangular or cylindrical input image.
#[must_use]
pub fn generate_cube_faces(
    src_image: &RgbImage,
    config: &TilerConfig,
    actual_cube_size: u32,
) -> Vec<(char, RgbImage)> {
    let (src_width, src_height) = src_image.dimensions();
    let haov_rad = config.angles.haov.to_radians();
    let vaov_rad = config.angles.vaov.to_radians();
    let v_offset_rad = config.angles.v_offset.to_radians();
    let interp_mode = config.output.interpolation_mode;

    // Convert normalized [0.0, 1.0] colors to Rgb<u8>
    let bg_color = Rgb(config.output.background_color);

    // Hugin standard camera orientation mapping
    let face_setups = ['f', 'b', 'u', 'd', 'l', 'r'];

    face_setups
        .into_par_iter()
        .map(|letter| {
            let mut face_img = RgbImage::new(actual_cube_size, actual_cube_size);

            let stride = actual_cube_size as usize * 3;
            let face_pixels: &mut [u8] = &mut face_img;

            // Direct mapping based on cardinal orientations
            let map_coords: fn(f64, f64) -> (f64, f64, f64) = match letter {
                'f' => |u, v| (u, -v, 1.0),   // front
                'b' => |u, v| (-u, -v, -1.0), // back
                'u' => |u, v| (u, 1.0, v),    // up
                'd' => |u, v| (u, -1.0, -v),  // down
                'l' => |u, v| (-1.0, -v, u),  // left
                'r' => |u, v| (1.0, -v, -u),  // right
                _ => unreachable!(),
            };

            face_pixels
                .par_chunks_exact_mut(stride)
                .enumerate()
                .for_each(|(row, row_pixels)| {
                    let v = ((row as f64 + 0.5) / f64::from(actual_cube_size)).mul_add(2.0, -1.0);
                    for col in 0..actual_cube_size {
                        let u = ((f64::from(col) + 0.5) / f64::from(actual_cube_size))
                            .mul_add(2.0, -1.0);

                        let (x2, y2, z2) = map_coords(u, v);

                        // Convert to a 3D unit direction ray
                        let length = z2.mul_add(z2, y2.mul_add(y2, x2 * x2)).sqrt();

                        // Map the 3D vector back to spherical coordinates (yaw/pitch angles)
                        let theta = x2.atan2(z2);
                        let phi = (y2 / length).clamp(-1.0, 1.0).asin();

                        let mut is_outside = false;

                        // Horizontal projection mapping
                        let src_x = if config.angles.haov >= 360.0 {
                            let normalized_theta = (theta + PI) / (2.0 * PI);
                            normalized_theta * f64::from(src_width)
                        } else {
                            let half_haov = haov_rad / 2.0;
                            if theta.abs() > half_haov {
                                is_outside = true;
                                0.0
                            } else {
                                let normalized_theta = f64::midpoint(theta / half_haov, 1.0);
                                normalized_theta * f64::from(src_width)
                            }
                        };

                        // Vertical projection mapping using angular offsets
                        let src_y = if is_outside {
                            0.0
                        } else {
                            match config.angles.projection {
                                Projection::Cylindrical => {
                                    let half_vaov = vaov_rad / 2.0;
                                    let max_y_cyl = half_vaov.tan();

                                    let phi_relative = phi - v_offset_rad;
                                    let y_cyl = phi_relative.tan();

                                    if y_cyl.abs() > max_y_cyl {
                                        is_outside = true;
                                        0.0
                                    } else {
                                        let normalized_y = y_cyl / max_y_cyl;
                                        (1.0 - normalized_y) / 2.0 * f64::from(src_height)
                                    }
                                }
                                Projection::Equirectangular => {
                                    if config.angles.vaov >= 180.0 {
                                        let normalized_phi = (FRAC_PI_2 - phi) / PI;
                                        normalized_phi * f64::from(src_height)
                                    } else {
                                        let half_vaov = vaov_rad / 2.0;

                                        // Offset the incoming ray pitch by the vertical center pitch of the crop
                                        let phi_relative = phi - v_offset_rad;

                                        if phi_relative.abs() > half_vaov {
                                            is_outside = true;
                                            0.0
                                        } else {
                                            let normalized_phi =
                                                f64::midpoint(phi_relative / half_vaov, 1.0);
                                            (1.0 - normalized_phi) * f64::from(src_height)
                                        }
                                    }
                                }
                            }
                        };

                        let pixel = if is_outside {
                            bg_color
                        } else {
                            match interp_mode {
                                InterpolationMode::Bicubic => sample_bicubic(
                                    src_image,
                                    src_x,
                                    src_y,
                                    config.angles.haov >= 360.0,
                                    bg_color,
                                ),
                                InterpolationMode::Bilinear => sample_bilinear(
                                    src_image,
                                    src_x,
                                    src_y,
                                    config.angles.haov >= 360.0,
                                    bg_color,
                                ),
                            }
                        };

                        let offset = col as usize * 3;
                        row_pixels[offset] = pixel[0];
                        row_pixels[offset + 1] = pixel[1];
                        row_pixels[offset + 2] = pixel[2];
                    }
                });

            (letter, face_img)
        })
        .collect()
}

fn sample_bilinear(img: &RgbImage, x: f64, y: f64, wrap_x: bool, bg: Rgb<u8>) -> Rgb<u8> {
    let (w, h) = img.dimensions();
    let w_f = f64::from(w);
    let h_f = f64::from(h);

    if y < 0.0 || y >= h_f || (!wrap_x && (x < 0.0 || x >= w_f)) {
        return bg;
    }

    let x_wrapped = if wrap_x { x.rem_euclid(w_f) } else { x };

    // Shift coordinates by -0.5 to align continuous mapping with pixel centers
    let x_center = x_wrapped - 0.5;
    let y_center = y - 0.5;

    let x0 = x_center.floor();
    let y0 = y_center.floor();

    let dx = x_center - x0;
    let dy = y_center - y0;

    let x0_i = x0 as i32;
    let y0_i = y0 as i32;

    let py0 = y0_i.clamp(0, h as i32 - 1) as u32;
    let py1 = (y0_i + 1).clamp(0, h as i32 - 1) as u32;

    // Helper to get wrapped X-coordinates
    let get_x_index = |px: i32| -> Option<u32> {
        if wrap_x {
            Some(px.rem_euclid(w as i32) as u32)
        } else if px < 0 || px >= w as i32 {
            None
        } else {
            Some(px as u32)
        }
    };

    let px0 = get_x_index(x0_i);
    let px1 = get_x_index(x0_i + 1);

    let p00 = px0.map_or(&bg, |x_idx| img.get_pixel(x_idx, py0));
    let p10 = px1.map_or(&bg, |x_idx| img.get_pixel(x_idx, py0));
    let p01 = px0.map_or(&bg, |x_idx| img.get_pixel(x_idx, py1));
    let p11 = px1.map_or(&bg, |x_idx| img.get_pixel(x_idx, py1));

    let w00 = (1.0 - dx) * (1.0 - dy);
    let w10 = dx * (1.0 - dy);
    let w01 = (1.0 - dx) * dy;
    let w11 = dx * dy;

    let r = f64::from(p11[0]).mul_add(
        w11,
        f64::from(p01[0]).mul_add(w01, f64::from(p10[0]).mul_add(w10, f64::from(p00[0]) * w00)),
    );
    let g = f64::from(p11[1]).mul_add(w11, f64::from(p01[1]).mul_add(w01, f64::from(p10[1]).mul_add(w10, f64::from(p00[1]) * w00)));
    let b = f64::from(p11[2]).mul_add(w11, f64::from(p01[2]).mul_add(w01, f64::from(p10[2]).mul_add(w10, f64::from(p00[2]) * w00)));

    Rgb([
        r.round().clamp(0.0, 255.0) as u8,
        g.round().clamp(0.0, 255.0) as u8,
        b.round().clamp(0.0, 255.0) as u8,
    ])
}

/// Helper function to perform bicubic sampling.
fn sample_bicubic(img: &RgbImage, x: f64, y: f64, wrap_x: bool, bg: Rgb<u8>) -> Rgb<u8> {
    let (w, h) = img.dimensions();
    let w_f = f64::from(w);
    let h_f = f64::from(h);

    if y < 0.0 || y >= h_f || (!wrap_x && (x < 0.0 || x >= w_f)) {
        return bg;
    }

    let x_wrapped = if wrap_x { x.rem_euclid(w_f) } else { x };

    // Shift coordinates by -0.5 to align continuous mapping with pixel centers
    let x_center = x_wrapped - 0.5;
    let y_center = y - 0.5;

    let x0 = x_center.floor();
    let y0 = y_center.floor();

    let dx = x_center - x0;
    let dy = y_center - y0;

    let x0_i = x0 as i32;
    let y0_i = y0 as i32;

    // Catmull-Rom cubic spline weights
    let get_weights = |t: f64| -> [f64; 4] {
        let t2 = t * t;
        let t3 = t2 * t;
        [
            0.5 * (2.0f64.mul_add(t2, -t3) - t),
            0.5 * (5.0f64.mul_add(-t2, 3.0 * t3) + 2.0),
            0.5 * (4.0f64.mul_add(t2, -3.0 * t3) + t),
            0.5 * (t3 - t2),
        ]
    };

    let wx = get_weights(dx);
    let wy = get_weights(dy);

    // Precalculate wrapped X-coordinates
    let mut px_mapped = [None; 4];
    for i in -1..=2 {
        let px = x0_i + i;
        px_mapped[(i + 1) as usize] = if wrap_x {
            Some(px.rem_euclid(w as i32) as u32)
        } else if px < 0 || px >= w as i32 {
            None
        } else {
            Some(px as u32)
        };
    }

    // Precalculate clamped Y-coordinates
    let mut py_clamped = [0u32; 4];
    for j in -1..=2 {
        let py = y0_i + j;
        py_clamped[(j + 1) as usize] = py.clamp(0, h as i32 - 1) as u32;
    }

    let mut r_sum = 0.0;
    let mut g_sum = 0.0;
    let mut b_sum = 0.0;

    for j in -1..=2 {
        let weight_y = wy[(j + 1) as usize];
        if weight_y == 0.0 {
            continue;
        }

        let py_c = py_clamped[(j + 1) as usize];

        for i in -1..=2 {
            let weight_x = wx[(i + 1) as usize];
            let weight = weight_x * weight_y;
            if weight == 0.0 {
                continue;
            }

            let pixel = px_mapped[(i + 1) as usize].map_or(&bg, |px_c| img.get_pixel(px_c, py_c));

            r_sum = f64::from(pixel[0]).mul_add(weight, r_sum);
            g_sum = f64::from(pixel[1]).mul_add(weight, g_sum);
            b_sum = f64::from(pixel[2]).mul_add(weight, b_sum);
        }
    }

    Rgb([
        r_sum.round().clamp(0.0, 255.0) as u8,
        g_sum.round().clamp(0.0, 255.0) as u8,
        b_sum.round().clamp(0.0, 255.0) as u8,
    ])
}
