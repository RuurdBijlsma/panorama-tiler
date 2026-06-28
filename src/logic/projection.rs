use crate::config::{GeneratorConfig, InterpolationMode, Projection};
use crate::logic::get_bg_color;
use image::{Rgb, RgbImage};
use rayon::prelude::*;
use std::f64::consts::{FRAC_PI_2, PI};

/// Generates the 6 cubemap face images from an equirectangular or cylindrical input image.
pub fn generate_cube_faces(
    src_image: &RgbImage,
    config: &GeneratorConfig,
    actual_cube_size: u32,
) -> Vec<(char, RgbImage)> {
    let (src_width, src_height) = src_image.dimensions();
    let haov_rad = config.partial_config.haov.to_radians();
    let vaov_rad = config.partial_config.vaov.to_radians();
    let v_offset_rad = config.partial_config.v_offset.to_radians();
    let interp_mode = config.interpolation_mode;

    // Convert normalized [0.0, 1.0] colors to Rgb<u8>
    let bg_color = get_bg_color(config);

    // Hugin standard camera orientation mapping
    let face_setups = vec![
        ('f', 0.0, 0.0),        // front
        ('b', PI, 0.0),         // back
        ('u', 0.0, -FRAC_PI_2), // up
        ('d', 0.0, FRAC_PI_2),  // down
        ('l', -FRAC_PI_2, 0.0), // left
        ('r', FRAC_PI_2, 0.0),  // right
    ];

    face_setups
        .into_par_iter()
        .map(|(letter, yaw, pitch)| {
            let mut face_img = RgbImage::new(actual_cube_size, actual_cube_size);

            let cos_p = pitch.cos();
            let sin_p = pitch.sin();
            let cos_y = yaw.cos();
            let sin_y = yaw.sin();

            let stride = actual_cube_size as usize * 3;
            let face_pixels: &mut [u8] = &mut face_img;

            face_pixels
                .par_chunks_exact_mut(stride)
                .enumerate()
                .for_each(|(row, row_pixels)| {
                    for col in 0..actual_cube_size {
                        let u = (col as f64 + 0.5) / actual_cube_size as f64 * 2.0 - 1.0;
                        let v = (row as f64 + 0.5) / actual_cube_size as f64 * 2.0 - 1.0;

                        let x_local = u;
                        let y_local = -v; // Invert because image rows increase downwards
                        let z_local = 1.0;

                        // 1. Pitch rotation (around X-axis) using precomputed trig
                        let x1 = x_local;
                        let y1 = y_local * cos_p - z_local * sin_p;
                        let z1 = y_local * sin_p + z_local * cos_p;

                        // 2. Yaw rotation (around Y-axis) using precomputed trig
                        let x2 = x1 * cos_y + z1 * sin_y;
                        let y2 = y1;
                        let z2 = -x1 * sin_y + z1 * cos_y;

                        // Convert to a 3D unit direction ray
                        let length = (x2 * x2 + y2 * y2 + z2 * z2).sqrt();
                        let x_dir = x2 / length;
                        let y_dir = y2 / length;
                        let z_dir = z2 / length;

                        // Map the 3D vector back to spherical coordinates (yaw/pitch angles)
                        let theta = x_dir.atan2(z_dir);
                        let phi = y_dir.asin();

                        let mut is_outside = false;

                        // Horizontal projection mapping
                        let src_x = if config.partial_config.haov >= 360.0 {
                            let normalized_theta = (theta + PI) / (2.0 * PI);
                            normalized_theta * (src_width as f64)
                        } else {
                            let half_haov = haov_rad / 2.0;
                            if theta.abs() > half_haov {
                                is_outside = true;
                                0.0
                            } else {
                                let normalized_theta = (theta / half_haov + 1.0) / 2.0;
                                normalized_theta * (src_width as f64)
                            }
                        };

                        // Vertical projection mapping using angular offsets
                        let src_y = if !is_outside {
                            match config.projection {
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
                                        (1.0 - normalized_y) / 2.0 * (src_height as f64)
                                    }
                                }
                                Projection::Equirectangular => {
                                    if config.partial_config.vaov >= 180.0 {
                                        let normalized_phi = (FRAC_PI_2 - phi) / PI;
                                        normalized_phi * (src_height as f64)
                                    } else {
                                        let half_vaov = vaov_rad / 2.0;

                                        // Offset the incoming ray pitch by the vertical center pitch of the crop
                                        let phi_relative = phi - v_offset_rad;

                                        if phi_relative.abs() > half_vaov {
                                            is_outside = true;
                                            0.0
                                        } else {
                                            let normalized_phi =
                                                (phi_relative / half_vaov + 1.0) / 2.0;
                                            (1.0 - normalized_phi) * (src_height as f64)
                                        }
                                    }
                                }
                            }
                        } else {
                            0.0
                        };

                        let pixel = if is_outside {
                            bg_color
                        } else {
                            match interp_mode {
                                InterpolationMode::Bicubic => sample_bicubic(
                                    src_image,
                                    src_x,
                                    src_y,
                                    config.partial_config.haov >= 360.0,
                                    bg_color,
                                ),
                                InterpolationMode::Bilinear => sample_bilinear(
                                    src_image,
                                    src_x,
                                    src_y,
                                    config.partial_config.haov >= 360.0,
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
    let w_f = w as f64;
    let h_f = h as f64;

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

    let get_pixel_helper = |px: i32, py: i32| -> &Rgb<u8> {
        let py_clamped = py.clamp(0, h as i32 - 1) as u32;
        if wrap_x {
            let px_wrapped = px.rem_euclid(w as i32) as u32;
            img.get_pixel(px_wrapped, py_clamped)
        } else if px < 0 || px >= w as i32 {
            &bg
        } else {
            img.get_pixel(px as u32, py_clamped)
        }
    };

    let p00 = get_pixel_helper(x0_i, y0_i);
    let p10 = get_pixel_helper(x0_i + 1, y0_i);
    let p01 = get_pixel_helper(x0_i, y0_i + 1);
    let p11 = get_pixel_helper(x0_i + 1, y0_i + 1);

    let w00 = (1.0 - dx) * (1.0 - dy);
    let w10 = dx * (1.0 - dy);
    let w01 = (1.0 - dx) * dy;
    let w11 = dx * dy;

    let r = p00[0] as f64 * w00 + p10[0] as f64 * w10 + p01[0] as f64 * w01 + p11[0] as f64 * w11;
    let g = p00[1] as f64 * w00 + p10[1] as f64 * w10 + p01[1] as f64 * w01 + p11[1] as f64 * w11;
    let b = p00[2] as f64 * w00 + p10[2] as f64 * w10 + p01[2] as f64 * w01 + p11[2] as f64 * w11;

    Rgb([
        r.round().clamp(0.0, 255.0) as u8,
        g.round().clamp(0.0, 255.0) as u8,
        b.round().clamp(0.0, 255.0) as u8,
    ])
}

/// Helper function to perform bicubic sampling.
fn sample_bicubic(img: &RgbImage, x: f64, y: f64, wrap_x: bool, bg: Rgb<u8>) -> Rgb<u8> {
    let (w, h) = img.dimensions();
    let w_f = w as f64;
    let h_f = h as f64;

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
            0.5 * (-t3 + 2.0 * t2 - t),
            0.5 * (3.0 * t3 - 5.0 * t2 + 2.0),
            0.5 * (-3.0 * t3 + 4.0 * t2 + t),
            0.5 * (t3 - t2),
        ]
    };

    let wx = get_weights(dx);
    let wy = get_weights(dy);

    let mut r_sum = 0.0;
    let mut g_sum = 0.0;
    let mut b_sum = 0.0;

    for j in -1..=2 {
        let py = y0_i + j;
        let weight_y = wy[(j + 1) as usize];
        if weight_y == 0.0 {
            continue;
        }

        let py_clamped = py.clamp(0, h as i32 - 1) as u32;

        for i in -1..=2 {
            let px = x0_i + i;
            let weight_x = wx[(i + 1) as usize];
            let weight = weight_x * weight_y;
            if weight == 0.0 {
                continue;
            }

            let pixel = if wrap_x {
                let px_wrapped = px.rem_euclid(w as i32) as u32;
                img.get_pixel(px_wrapped, py_clamped)
            } else if px < 0 || px >= w as i32 {
                &bg
            } else {
                img.get_pixel(px as u32, py_clamped)
            };

            r_sum += pixel[0] as f64 * weight;
            g_sum += pixel[1] as f64 * weight;
            b_sum += pixel[2] as f64 * weight;
        }
    }

    Rgb([
        r_sum.round().clamp(0.0, 255.0) as u8,
        g_sum.round().clamp(0.0, 255.0) as u8,
        b_sum.round().clamp(0.0, 255.0) as u8,
    ])
}
