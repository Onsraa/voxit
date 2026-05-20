// Shared bench fixtures. Each bench file `mod common`s this with #[path =
// "common.rs"]. Synthetic heightmaps so benches don't depend on the on-disk
// fixture.

use voxit::source::{RawVolume, ThresholdConfig};
use voxit::ui::resources::{BiomeMode, PreviewSettings};
use voxit::visibility::VisibilityMask;
use voxit::volume::{build_from_geotiff, VoxelGrid};

const SECS_PER_DEG: f32 = 3600.0;

pub fn synthetic_raw(side: u32, peak_amp_m: f32) -> RawVolume {
    let total = (side * side) as usize;
    let mut data = Vec::with_capacity(total);
    let sf = side as f32;
    for y in 0..side {
        for x in 0..side {
            let fx = (x as f32) / sf;
            let fy = (y as f32) / sf;
            let pattern = (fx * 8.0).sin() * (fy * 6.0).cos();
            let elev = peak_amp_m * 0.5 + peak_amp_m * 0.5 * pattern;
            data.push(elev);
        }
    }
    RawVolume {
        data,
        dims: [side, side, 1],
        spacing: [1.0 / SECS_PER_DEG, 1.0 / SECS_PER_DEG, 1.0],
        origin: [0.0, 0.0, 0.0],
    }
}

pub fn settings_for(grid: &VoxelGrid) -> PreviewSettings {
    PreviewSettings {
        density_m_per_voxel: grid.density,
        threshold_min: grid.elev_min,
        threshold_max: grid.elev_min + grid.density * grid.dims[1] as f32,
        elev_full_min: grid.elev_min,
        elev_full_max: grid.elev_min + grid.density * grid.dims[1] as f32,
        crop_x: [0.0, 1.0],
        crop_y: [0.0, 1.0],
        crop_z: [0.0, 1.0],
        grid_dims: grid.dims,
        sea_level_m: grid.elev_min,
        vertical_exaggeration: grid.vertical_exaggeration,
        biome_mode: BiomeMode::Elevation,
    }
}

/// Build a grid at a target output cell count by picking a `density` that
/// keeps grid.dims[0] ≈ target_side. Returns (grid, settings, mask).
pub fn fixture(raw_side: u32, density: f32, peak_amp_m: f32) -> (VoxelGrid, PreviewSettings, VisibilityMask) {
    let raw = synthetic_raw(raw_side, peak_amp_m);
    let thresholds = ThresholdConfig {
        min: 0.0,
        max: peak_amp_m,
    };
    let grid = build_from_geotiff(&raw, density, 1.0, &thresholds);
    let settings = settings_for(&grid);
    let mask = VisibilityMask::compute(&grid, &settings);
    (grid, settings, mask)
}

pub fn raw_for(raw_side: u32, peak_amp_m: f32) -> (RawVolume, ThresholdConfig) {
    let raw = synthetic_raw(raw_side, peak_amp_m);
    let thresholds = ThresholdConfig {
        min: 0.0,
        max: peak_amp_m,
    };
    (raw, thresholds)
}
