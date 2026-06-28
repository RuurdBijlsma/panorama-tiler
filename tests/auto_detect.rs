use pano_tiler::tile_panorama_file;
use std::path::Path;

#[test]
fn test_auto_tiling() {
    let input = Path::new("img/sphere/PXL_20220918_115954889.PHOTOSPHERE.jpg");
    let out_folder = format!(
        "target/auto_tile_{}",
        input.file_name().unwrap().to_string_lossy().to_string()
    );
    let output = Path::new(&out_folder);

    tile_panorama_file(input, output, None)
        .expect("Automatic metadata configuration and tiling failed");

    assert!(output.join("config.json").exists());
}
