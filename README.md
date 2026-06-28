

    let aspect_ratio = (width as f64) / (height as f64);
    let is_full_pano = config.angles.projection == Projection::Cylindrical || (aspect_ratio - 2.0).abs() < 1e-3;

    let haov =
        config.angles.haov
            .or_else(|| is_full_pano.then_some(360.0))
            .ok_or_else(|| TilerError::InvalidConfig("If `haov` is None, the input image must be a full (not partial) panorama!".to_string()))?;
    let vaov =
        config.angles.vaov
            .or_else(|| is_full_pano.then_some(180.0))
            .ok_or_else(|| TilerError::InvalidConfig("If `vaov` is None, the input image must be a full (not partial) panorama!".to_string()))?;