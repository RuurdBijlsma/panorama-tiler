/// Derived mathematical angles of view for a partial panorama.
#[derive(Debug, Clone, Copy)]
pub struct DerivedAngles {
    pub haov: f64,
    pub vaov: f64,
}

/// Calculates the proper horizontal (haov) and vertical (vaov) angles of view
/// from an image's dimensions and its 35mm equivalent focal length.
///
/// * `focal_length_35mm_eq` - Focal length in 35mm equivalent
/// * `width` - The width of the stitched image in pixels
/// * `height` - The height of the stitched image in pixels
/// * `crop_factor` - The estimated portion of the sensor height preserved after alignment.
///   Typically, 0.90 (90%) for standard sweeps.
pub fn calculate_pano_angles(
    focal_length_35mm_eq: f64,
    width: u32,
    height: u32,
    crop_factor: f64,
) -> Option<DerivedAngles> {
    if focal_length_35mm_eq <= 0.0 || width == 0 || height == 0 {
        return None;
    }

    // Calculate uncropped vertical field of view (VFOV) of the lens.
    // VFOV = 2 * arctan(12.0 / focal_length)
    let v_fov_lens_rad = 2.0 * (12.0 / focal_length_35mm_eq).atan();
    let v_fov_lens_deg = v_fov_lens_rad.to_degrees();

    // Adjust for vertical height lost during frame alignment drift (crop_factor)
    let vaov = v_fov_lens_deg * crop_factor;
    let vaov_rad = vaov.to_radians();

    // Derive HAOV based on cylindrical projection aspect ratio math:
    // AspectRatio = haov / (2 * tan(vaov/2))
    let aspect_ratio = width as f64 / height as f64;
    let haov_rad = aspect_ratio * 2.0 * (vaov_rad / 2.0).tan();
    let haov = haov_rad.to_degrees();

    Some(DerivedAngles { haov, vaov })
}
