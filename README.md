# voxel-viewer

Bevy desktop app that converts **DICOM medical scans** or **GeoTIFF elevation
tiles** into **MagicaVoxel `.vox` files** with a live 3D preview.

Adjust voxel density, threshold, and crop with sliders. Export a single-model
or chunked multi-model `.vox` consumable by MagicaVoxel or a downstream
voxel-aware engine.

## Status

Phase 0 — project boot. See `TODO.md` for the roadmap and current phase.

## Build

```bash
cargo run --release
```

## Documentation

- `docs/DESIGN.md` — full design spec
- `TODO.md` — phased roadmap
- `CLAUDE.md` — context for AI-assisted development

## License

TBD.
