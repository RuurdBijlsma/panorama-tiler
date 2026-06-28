use crate::exif::{PanoExif, calc_cylindrical_pano_angles, exif_to_partial_pano_config};
use crate::{PanoAngles, Projection, TilerError};
use exif::Exif;
use std::fs::File;
use std::path::Path;
use xmpkit::XmpMeta;

fn get_exif_metadata(file: &Path) -> Option<Exif> {
    let Ok(file) = File::open(file) else {
        return None;
    };
    let mut buf_reader = std::io::BufReader::new(file);
    let exif_reader = exif::Reader::new();
    exif_reader.read_from_container(&mut buf_reader).ok()
}

fn get_xmp_metadata(file: &Path) -> Option<XmpMeta> {
    let mut xmp_file = xmpkit::XmpFile::new();
    xmp_file
        .open(file)
        .ok()
        .and_then(|_| xmp_file.get_xmp())
        .cloned()
}

fn get_dimensions(file: &Path, exif: Option<&Exif>) -> Result<(u32, u32), TilerError> {
    if let Some(exif) = exif {
        let width = exif.get_field(exif::Tag::PixelXDimension, exif::In::PRIMARY)
            .or_else(|| exif.get_field(exif::Tag::ImageWidth, exif::In::PRIMARY))
            .and_then(|f| f.value.get_uint(0));
        let height = exif.get_field(exif::Tag::PixelYDimension, exif::In::PRIMARY)
            .or_else(|| exif.get_field(exif::Tag::ImageLength, exif::In::PRIMARY))
            .and_then(|f| f.value.get_uint(0));
        if let (Some(w), Some(h)) = (width, height) {
            return Ok((w, h));
        }
    }
    Ok(image::image_dimensions(file)?)
}

pub fn guess_pano_angles(file: &Path) -> Result<PanoAngles, TilerError> {
    //  Extract metadata
    let xmp_metadata = get_xmp_metadata(file);

    // Query Google Photo Sphere (GPano) values
    let north_offset = if let Some(meta) = &xmp_metadata {
        let gpano_ns = "http://ns.google.com/photos/1.0/panorama/";

        // Helper to query and safely convert arbitrary XmpValue stringified forms
        let get_gpano_f64 = |name: &str| -> Option<f64> {
            meta.get_property(gpano_ns, name)
                .and_then(|v| v.to_string().parse::<f64>().ok())
        };

        let projection_type = meta
            .get_property(gpano_ns, "ProjectionType")
            .map(|v| v.to_string());
        let cropped_area_height = get_gpano_f64("CroppedAreaImageHeightPixels");
        let cropped_area_width = get_gpano_f64("CroppedAreaImageWidthPixels");
        let full_pano_height = get_gpano_f64("FullPanoHeightPixels");
        let full_pano_width = get_gpano_f64("FullPanoWidthPixels");
        let cropped_area_top = get_gpano_f64("CroppedAreaTopPixels");
        let pose_heading = get_gpano_f64("PoseHeadingDegrees");

        // Check if we have complete partial photo sphere crop boundaries
        if let (Some(cropped_w), Some(cropped_h), Some(full_w), Some(full_h), Some(cropped_t)) = (
            cropped_area_width,
            cropped_area_height,
            full_pano_width,
            full_pano_height,
            cropped_area_top,
        ) {
            // Build temporary EXIF container to derive consistent angles and offsets
            let exif_info = PanoExif {
                full_pano_width_pixels: full_w as u32,
                full_pano_height_pixels: full_h as u32,
                cropped_area_top_pixels: cropped_t as u32,
                cropped_area_image_width_pixels: cropped_w as u32,
                cropped_area_image_height_pixels: cropped_h as u32,
                projection_type,
                pose_heading_degrees: pose_heading,
            };

            return Ok(exif_to_partial_pano_config(&exif_info));
        }

        pose_heading
    } else{
        None
    };

    // Initialize detection variables
    let mut haov = None;
    let mut vaov = None;
    let mut projection = Projection::Equirectangular;

    // 6. Cylindrical Sweep detection via focal length EXIF tags
    let exif_metadata = get_exif_metadata(file);
    let mut focal_length_35mm = None;

    if let Some(exif) = &exif_metadata
        && let Some(field) = exif.get_field(exif::Tag::FocalLengthIn35mmFilm, exif::In::PRIMARY)
    {
        focal_length_35mm = match &field.value {
            exif::Value::Rational(rationals) => rationals.first().map(|r| r.to_f64()),
            _ => field.value.get_uint(0).map(|v| v as f64),
        };
    }

    let (width, height) = get_dimensions(file, exif_metadata.as_ref())?;
    if let Some(focal) = focal_length_35mm
        && focal > 0.0
    {
        projection = Projection::Cylindrical;
        let crop_factor = 0.90; // Standard crop loss ratio for panorama stitches
        if let Some(angles) = calc_cylindrical_pano_angles(focal, width, height, crop_factor) {
            haov = Some(angles.haov);
            vaov = Some(angles.vaov);
        }
    }

    // 7. Fallback heuristics for un-tagged panoramic images
    let aspect_ratio = width as f64 / height as f64;
    if haov.is_none() && vaov.is_none() {
        if (aspect_ratio - 2.0).abs() <= 0.1 {
            // Equirectangular Full Photo Sphere
            projection = Projection::Equirectangular;
            haov = Some(360.0);
            vaov = Some(180.0);
        } else if aspect_ratio >= 2.2 {
            // Wide image treated as a custom cylindrical sweep (e.g. 24mm default sweep equivalent)
            projection = Projection::Cylindrical;
            let crop_factor = 0.90;
            if let Some(angles) = calc_cylindrical_pano_angles(24.0, width, height, crop_factor) {
                haov = Some(angles.haov);
                vaov = Some(angles.vaov);
            }
        } else {
            // Default equirectangular
            projection = Projection::Equirectangular;
            haov = Some(360.0);
            vaov = Some(180.0);
        }
    }

    Ok(PanoAngles {
        haov: haov.unwrap_or(360.0),
        vaov: vaov.unwrap_or(180.0),
        north_offset,
        projection,
        v_offset: 0.0,
        horizon_pixels: 0,
    })
}
