use pano_tiler::{OutputConfig, OutputFormat, tile_panorama_with_guessed_angles};
use std::path::Path;

#[test]
fn test_auto_tiling() {
    let paths = &[
        Path::new("img/cylinder/PXL_20260414_113245071.PANO.jpg"),
        Path::new("img/partial_sphere/PXL_20210730_183041272.PANO.jpg"),
        Path::new("img/partial_sphere/PANO_20130622_091214.jpg"),
        Path::new("img/partial_sphere/PANO_20200806_210426.jpg"),
        Path::new("img/partial_sphere/PANO_20210207_152515.jpg"),
        Path::new("img/partial_sphere/PXL_20210730_144252204.PHOTOSPHERE.jpg"),
        Path::new("img/sphere/PXL_20210722_151141413.PHOTOSPHERE.jpg"),
        Path::new("img/sphere/PXL_20220918_115954889.PHOTOSPHERE.jpg"),
    ];
    for path in paths {
        do_auto_tiling(path);
    }
}

fn do_auto_tiling(path: &Path) {
    let out_folder = format!(
        "target/auto_tile_{}",
        path.file_name().unwrap().to_string_lossy()
    );
    let output = Path::new(&out_folder);

    tile_panorama_with_guessed_angles(
        path,
        output,
        Some(OutputConfig {
            format: OutputFormat::Webp,
            quality: 85,
            ..Default::default()
        }),
    )
    .expect("Automatic metadata configuration and tiling failed");

    assert!(output.join("config.json").exists());
}
