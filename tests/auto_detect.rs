use pano_tiler::tile_panorama_with_guessed_angles;
use std::path::Path;

#[test]
fn test_auto_tiling() {
    let input = Path::new("img/cylinder/PXL_20260414_113245071.PANO.jpg");
    let out_folder = format!(
        "target/auto_tile_{}",
        input.file_name().unwrap().to_string_lossy()
    );
    let output = Path::new(&out_folder);

    tile_panorama_with_guessed_angles(input, output, None)
        .expect("Automatic metadata configuration and tiling failed");

    assert!(output.join("config.json").exists());
}
