use crate::config::{Projection, TilerConfig};
use image::{Rgb, RgbImage};
use rayon::prelude::*;
use std::f64::consts::{FRAC_PI_2, PI};

/// Generates the 6 cubemap face images from an equirectangular or cylindrical input image.
pub fn generate_cube_faces(
    src_image: &RgbImage,
    config: &TilerConfig,
    actual_cube_size: u32,
) -> Vec<(char, RgbImage)> {
    let (src_width, src_height) = src_image.dimensions();
    let haov_rad = config.partial_config.haov.to_radians();
    let vaov_rad = config.partial_config.vaov.to_radians();
    let horizon_pixels = config.partial_config.horizon_pixels;

    // Convert normalized [0.0, 1.0] colors to Rgb<u8>
    let bg_color = Rgb([
        (config.partial_config.background_color[0] * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8,
        (config.partial_config.background_color[1] * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8,
        (config.partial_config.background_color[2] * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8,
    ]);

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

            for row in 0..actual_cube_size {
                for col in 0..actual_cube_size {
                    let u = (col as f64 + 0.5) / actual_cube_size as f64 * 2.0 - 1.0;
                    let v = (row as f64 + 0.5) / actual_cube_size as f64 * 2.0 - 1.0;

                    let x_local = u;
                    let y_local = -v; // Invert because image rows increase downwards
                    let z_local = 1.0;

                    // 1. Pitch rotation (around X-axis)
                    let cos_p = pitch.cos();
                    let sin_p = pitch.sin();
                    let x1 = x_local;
                    let y1 = y_local * cos_p - z_local * sin_p;
                    let z1 = y_local * sin_p + z_local * cos_p;

                    // 2. Yaw rotation (around Y-axis)
                    let cos_y = yaw.cos();
                    let sin_y = yaw.sin();
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

                    // Vertical projection mapping
                    let src_y = if !is_outside {
                        match config.projection {
                            Projection::Cylindrical => {
                                let half_vaov = vaov_rad / 2.0;
                                let max_y_cyl = half_vaov.tan();
                                let y_cyl = phi.tan();

                                if y_cyl.abs() > max_y_cyl {
                                    is_outside = true;
                                    0.0
                                } else {
                                    let normalized_y = y_cyl / max_y_cyl;
                                    let y_base = (1.0 - normalized_y) / 2.0 * (src_height as f64);
                                    y_base + (horizon_pixels as f64)
                                }
                            }
                            Projection::Equirectangular => {
                                if config.partial_config.vaov >= 180.0 {
                                    let normalized_phi = (FRAC_PI_2 - phi) / PI;
                                    let y_base = normalized_phi * (src_height as f64);
                                    y_base + (horizon_pixels as f64)
                                } else {
                                    let half_vaov = vaov_rad / 2.0;

                                    if phi.abs() > half_vaov {
                                        is_outside = true;
                                        0.0
                                    } else {
                                        let normalized_phi = (phi / half_vaov + 1.0) / 2.0;
                                        let y_base =
                                            (1.0 - normalized_phi) / 2.0 * (src_height as f64);
                                        y_base + (horizon_pixels as f64)
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
                        sample_bilinear(
                            src_image,
                            src_x,
                            src_y,
                            config.partial_config.haov >= 360.0,
                            bg_color,
                        )
                    };

                    face_img.put_pixel(col, row, pixel);
                }
            }
            (letter, face_img)
        })
        .collect()
}

/// Helper function to perform standard bilinear sampling.
fn sample_bilinear(img: &RgbImage, x: f64, y: f64, wrap_x: bool, bg: Rgb<u8>) -> Rgb<u8> {
    let (w, h) = img.dimensions();
    let w_f = w as f64;
    let h_f = h as f64;

    if y < 0.0 || y >= h_f || (!wrap_x && (x < 0.0 || x >= w_f)) {
        return bg;
    }

    let x_wrapped = if wrap_x { x.rem_euclid(w_f) } else { x };

    let x0 = x_wrapped.floor();
    let y0 = y.floor();
    let x1 = if wrap_x {
        (x0 + 1.0) % w_f
    } else {
        (x0 + 1.0).min(w_f - 1.0)
    };
    let y1 = (y0 + 1.0).min(h_f - 1.0);

    let dx = x_wrapped - x0;
    let dy = y - y0;

    let p00 = img.get_pixel(x0 as u32, y0 as u32);
    let p10 = img.get_pixel(x1 as u32, y0 as u32);
    let p01 = img.get_pixel(x0 as u32, y1 as u32);
    let p11 = img.get_pixel(x1 as u32, y1 as u32);

    let mut out = [0u8; 3];
    for c in 0..3 {
        let val = (1.0 - dx) * (1.0 - dy) * p00[c] as f64
            + dx * (1.0 - dy) * p10[c] as f64
            + (1.0 - dx) * dy * p01[c] as f64
            + dx * dy * p11[c] as f64;
        out[c] = val.round().clamp(0.0, 255.0) as u8;
    }
    Rgb(out)
}
