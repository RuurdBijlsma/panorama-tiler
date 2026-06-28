use crate::{OutputFormat, TilerError};
use image::RgbImage;
use image::codecs::jpeg::JpegEncoder;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub fn save_image(
    image: &RgbImage,
    filepath: &Path,
    output_format: OutputFormat,
    quality: u8,
) -> Result<(), TilerError> {
    match output_format {
        OutputFormat::Png => {
            image.save(filepath)?;
        }
        OutputFormat::Jpeg => {
            let file = File::create(filepath)?;
            let mut writer = BufWriter::new(file);
            let mut encoder = JpegEncoder::new_with_quality(&mut writer, quality);
            encoder.encode_image(image)?;
        }
        #[cfg(feature = "webp")]
        OutputFormat::Webp => {
            let (width, height) = image.dimensions();
            let encoder = webp::Encoder::from_rgb(image.as_raw(), width, height);
            let encoded_webp = encoder.encode(quality as f32);
            fs::write(filepath, &*encoded_webp)?;
        }
    }
    Ok(())
}
