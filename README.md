# panorama-tiler

Tiling and configuration generator for creating multi-resolution cubemap pyramids from equirectangular or cylindrical
panoramas, designed for use with the [Pannellum](https://pannellum.org/) web viewer.

[![Crates.io](https://img.shields.io/crates/v/panorama-tiler.svg)](https://crates.io/crates/panorama-tiler)
[![Documentation](https://docs.rs/panorama-tiler/badge.svg)](https://docs.rs/panorama-tiler)

![panorama_tiler_concept.jpg](.github/img_header.jpg)

`panorama-tiler` transforms flat equirectangular or cylindrical panorama images into partitioned, multi-resolution zoom
levels and outputs a companion `config.json` configuration file. It supports both complete 360-degree spheres and
cropped/partial panoramas.

## Features

- **Projection Conversion**: Converts equirectangular and cylindrical source projections into 6-faced cubemaps (`f`,
  `b`, `u`, `d`, `l`, `r`).
- **Angle and Crop Extraction**: Parses Google Photo Sphere (GPano) XMP tags and EXIF data to determine field of view (
  HAOV/VAOV) and vertical alignment.
- **High resolution**: Pannellum multi-res supports high resolution panoramas by splitting the image into tiles, this
  crate does the tiling.
- **Pyramid Downscaling**: Generates multi-level zoom crops of the panorama that Pannellum loads dynamically.
- **Fast**: Over 10x faster than Pannellum's `generate.py` for (almost) the same functionality.
- **No `Hugin`/`nona` dependency** This is a pure rust implemenation, no external dependencies needed.

---

## What's this?

This crate acts as a pre-processing pipeline for Pannellum's **Multi-resolution (`multires`)** panorama type. Rather
than loading a single high-resolution image, which can degrade web client performance or hit browser texture limits,
this crate segments the panorama into smaller tiles (e.g., 512x512 pixels) across different resolution tiers. The tile
logic is similar to how map tiles work, given that it generates tiles for different zoom levels until the full
resolution of the panorama is reached.

#### What it can do:

- Generate directory hierarchies containing cropped tiles for each zoom tier (`/%l/%s%y_%x`).
- Generate flat, low-resolution fallback cubemaps for fallback paths (`/fallback/%s`).
- Compute and output structural constraints to `config.json`, including:
    - `haov`, `minYaw`, `maxYaw`
    - `vaov`, `minPitch`, `maxPitch`, `vOffset`
    - `northOffset`
    - `multiRes` object parameters (`tileResolution`, `maxLevel`, `cubeResolution`, `missingTiles`, `path`,
      `fallbackPath`)
- Compress missing tile grids for partial views using Pannellum's Base83 character set specification.

#### What it cannot do:

- It does not generate a perceptual hash, like `generate.py` does
- It does not stitch overlapping source images together. The input image must already be a fully stitched panorama.

---

## Core Concepts

- **Equirectangular vs. Cylindrical**: Equirectangular projection maps coordinates linearly across latitude and
  longitude. Cylindrical projection maps coordinates onto a cylinder, requiring trigonometric scaling of vertical
  coordinates based on focal length. An equirectangular panorama you might be familiar with is Google StreetView.
- **Direct vs. Recursive Downscaling**:
    - `Direct`: Each pyramid tier is scaled down directly from the full-resolution cubemap face. This preserves image
      sharpness but takes longer.
    - `Recursive`: Tiers are recursively generated from the level immediately above them. This is faster but can
      introduce minor interpolation errors over deep zoom pyramids.
- **Base83 Missing Tiles String**: For partial panoramas, tiles that contain purely background color are discarded
  during processing. To prevent Pannellum from throwing 404 HTTP errors trying to fetch these non-existent tiles, their
  absence is encoded into a Base83 string stored in `config.json`.

---

## Usage

### Automatic Parameter Extraction

If your input image contains Exif/XMP metadata for panoramas, you can run the auto-detect pipeline. If it can't find the
proper tags it will do a best guess estimation to find the proper configuration.

```rust
use panorama_tiler::{OutputConfig, OutputFormat, tile_panorama_with_guessed_angles};
use std::path::Path;

fn main() -> Result<(), panorama_tiler::TilerError> {
    let input_path = Path::new("img/sphere/input_photosphere.jpg");
    let output_dir = Path::new("tiles_output");

    let output_config = OutputConfig {
        format: OutputFormat::Webp, // Supports Jpeg, Png, Webp
        quality: 85,
        ..Default::default()
    };

    tile_panorama_with_guessed_angles(
        input_path,
        output_dir,
        Some(output_config),
    )?;

    Ok(())
}
```

### Manual Configuration

When metadata tags are absent or you need to supply manual field-of-view angles, you can define them explicitly.

```rust
use panorama_tiler::{
    TilerConfig, PanoAngles, OutputConfig, Projection, InterpolationMode,
    DownscalingMethod, tile_panorama, OutputFormat
};
use std::path::Path;

fn main() -> Result<(), panorama_tiler::TilerError> {
    let config = TilerConfig {
        angles: PanoAngles {
            haov: 180.0,             // Horizontal Angle of View (degrees)
            vaov: 90.0,              // Vertical Angle of View (degrees)
            v_offset: 5.0,           // Vertical center pitch offset
            projection: Projection::Cylindrical,
            ..Default::default()
        },
        output: OutputConfig {
            tile_size: 512,
            fallback_size: 1024,
            format: OutputFormat::Jpeg,
            quality: 80,
            interpolation_mode: InterpolationMode::Bicubic,
            downscaling_method: DownscalingMethod::Direct,
            ..Default::default()
        },
    };

    tile_panorama(
        Path::new("img/cylinder/input_pano.jpg"),
        Path::new("target/tiles_output"),
        &config,
    )?;

    Ok(())
}
```

---

## Minimal Frontend Example

Once the tile pyramid is generated, host the target directory containing your tiles and the `config.json` file. You can
then load it into Pannellum with the following client-side HTML:

```html
<!DOCTYPE html>
<html lang="en">

<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Multires Panorama</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/pannellum@2.5.7/build/pannellum.css">
  <script src="https://cdn.jsdelivr.net/npm/pannellum@2.5.7/build/pannellum.js"></script>
  <style>
    body {
      margin: 0;
    }

    #panorama {
      width: 100vw;
      height: 100vh;
    }
  </style>
</head>

<body>

<div id="panorama"></div>

<script>
  // Point `BASE_PATH` to the output folder generated
  const BASE_PATH = 'tiles_output'
  fetch(`${BASE_PATH}/config.json`)
          .then(response => response.json())
          .then(config => {
            config.multiRes.basePath = BASE_PATH
            config.autoLoad = true
            if (config.multiRes.fallbackPath) {
              config.multiRes.fallbackPath = 'tiles_output' + config.multiRes.fallbackPath;
            }
            pannellum.viewer('panorama', config);
          });
</script>

</body>

</html>
```

---

## Cargo Features

| Feature    | Default | Description                                              | Dependencies     |
|:-----------|:-------:|:---------------------------------------------------------|:-----------------|
| `metadata` | **Yes** | Enables metadata reading, parsing GPano and EXIF fields. | `exif`, `xmpkit` |
| `webp`     | **Yes** | Enables saving processed tiles in WebP format.           | `webp`           |

---

## Performance Measurements

The pipeline is implemented using Rayon to compute cubemap pixel mappings and encode target tiles in parallel. Below is
an indicative execution time comparison between a typical single-threaded Python pre-processing script (`generate.py`)
and this Rust implementation on a standardized 4000x2000 equirectangular source image:

| Metric                 | Python Script (`generate.py`) | Rust Implementation |
|:-----------------------|:------------------------------|:--------------------|
| **Mean Execution**     | 10.380 seconds                | 1.031 seconds       |
| **Median Execution**   | 10.097 seconds                | 1.028 seconds       |
| **Standard Deviation** | 1.121 seconds                 | 0.008 seconds       |