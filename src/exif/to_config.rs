use crate::{PanoAngles, Projection};

pub struct PanoExif {
    pub projection_type: Option<String>,
    pub pose_heading_degrees: Option<f64>,
    pub full_pano_width_pixels: u32,
    pub full_pano_height_pixels: u32,
    pub cropped_area_top_pixels: u32,
    pub cropped_area_image_width_pixels: u32,
    pub cropped_area_image_height_pixels: u32,
}

#[must_use]
pub fn exif_to_partial_pano_config(exif_info: &PanoExif) -> PanoAngles {
    let cropped_area_img_width_pixels = f64::from(exif_info.cropped_area_image_width_pixels);
    let cropped_area_image_height_pixels = f64::from(exif_info.cropped_area_image_height_pixels);
    let full_pano_width_pixels = f64::from(exif_info.full_pano_width_pixels);
    let full_pano_height_pixels = f64::from(exif_info.full_pano_height_pixels);
    let cropped_area_top_pixels = f64::from(exif_info.cropped_area_top_pixels);

    // Angular views
    let haov = (cropped_area_img_width_pixels / full_pano_width_pixels) * 360.0;
    let vaov = (cropped_area_image_height_pixels / full_pano_height_pixels) * 180.0;

    // Vertical offset calculation
    let crop_center_y = cropped_area_top_pixels + (cropped_area_image_height_pixels / 2.0);
    let v_offset = -((crop_center_y / full_pano_height_pixels) - 0.5) * 180.0;

    // Horizon pixel offset (within cropped image space)
    let horizon_y_crop = (full_pano_height_pixels / 2.0) - cropped_area_top_pixels;
    let center_y_crop = cropped_area_image_height_pixels / 2.0;
    let horizon_pixels = (horizon_y_crop - center_y_crop).round() as i32;

    let projection = exif_info
        .projection_type
        .as_deref()
        .map_or_else(Projection::default, |pt| {
            if pt.trim().eq_ignore_ascii_case("cylindrical") {
                Projection::Cylindrical
            } else {
                Projection::Equirectangular
            }
        });

    PanoAngles {
        haov,
        vaov,
        v_offset,
        horizon_pixels,
        north_offset: exif_info.pose_heading_degrees,
        projection,
    }
}

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
#[must_use]
pub fn calc_cylindrical_pano_angles(
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
    let aspect_ratio = f64::from(width) / f64::from(height);
    let haov_rad = aspect_ratio * 2.0 * (vaov_rad / 2.0).tan();
    let haov = haov_rad.to_degrees();

    Some(DerivedAngles { haov, vaov })
}
