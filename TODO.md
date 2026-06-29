# TODO

* ✅ output to webp, jpg, png
* ✅ cargo feature voor webp output
* ✅ benchmarks
* ✅ defaults for config
* ✅ improve performance
* ✅ fast_image_resize
* ✅ test with partial spherical pano? (photosphere)
* ✅ exif helpers?
* ✅ refactor this garbage
    * ✅ Have exif extractor be standalone thing, not a do-it-all function
    * ✅ clean up helpers there's too many spread around
    * ✅ orchestrator.rs is too large and badly named
    * ✅ clear up if focal length based extraction is cylindrical only, if so name the file/function properly
    * ✅ add function to gen pano and save to disc in 1
* ✅ check voor panic possibilities
* ✅ gate exif extractor (xmpkit & exif) behind cargo feature
* ✅ what do i do with "view-layer" fields like "compass" to the pannellum config. (they are set in frontend)
* write readme
