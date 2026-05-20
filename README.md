# voxit

Drop a DICOM scan or GeoTIFF elevation tile, voxelize it live, export a MagicaVoxel `.vox`.

Bevy 0.14 desktop app. Pan-orbit 3D preview. Side panel sliders: voxel size, threshold, sea level, crop X/Y/Z, vertical exaggeration. Single-model export today; chunked multi-model and DICOM source come later.

## Run

```bash
cargo run --release
```

Drag a `.tif` onto the window or pick one. Tweak. Click Export `.vox`.

## License

TBD.
