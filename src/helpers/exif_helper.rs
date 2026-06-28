use crate::PartialPanoConfig;

pub struct PanoExif {
    pub full_pano_width_pixels: u32,
    pub full_pano_height_pixels: u32,
    pub cropped_area_top_pixels: u32,
    pub cropped_area_image_width_pixels: u32,
    pub cropped_area_image_height_pixels: u32,
}

pub fn exif_to_partial_pano_config(exif_info: &PanoExif) -> PartialPanoConfig {
    let cropped_area_img_width_pixels = exif_info.cropped_area_image_width_pixels as f64;
    let cropped_area_image_height_pixels = exif_info.cropped_area_image_height_pixels as f64;
    let full_pano_width_pixels = exif_info.full_pano_width_pixels as f64;
    let full_pano_height_pixels = exif_info.full_pano_height_pixels as f64;
    let cropped_area_top_pixels = exif_info.cropped_area_top_pixels as f64;

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

    PartialPanoConfig {
        haov,
        vaov,
        v_offset,
        horizon_pixels,
    }
}
