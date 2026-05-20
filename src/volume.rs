use bevy::prelude::*;

use crate::classify::classify_band;
use crate::source::{RawVolume, SourceKind, ThresholdConfig};

#[derive(Resource, Debug)]
pub struct VoxelGrid {
    pub data: Vec<u8>,
    pub dims: [u32; 3],
    pub density: f32,
    pub elev_min: f32,
    pub vertical_exaggeration: f32,
}

impl VoxelGrid {
    pub fn index(&self, x: u32, y: u32, z: u32) -> usize {
        (x as usize)
            + (self.dims[0] as usize) * (y as usize)
            + (self.dims[0] as usize) * (self.dims[1] as usize) * (z as usize)
    }

    pub fn get(&self, x: u32, y: u32, z: u32) -> u8 {
        self.data[self.index(x, y, z)]
    }
}

// Approx meters per raw pixel along the longest horizontal axis. Phase 5 assumes
// EPSG:4326 (degrees) — spacing × ~111 km/deg. Projected CRS support is Phase 8+.
const METERS_PER_DEGREE: f32 = 111_319.0;

pub fn estimate_pixel_spacing_m(raw: &RawVolume) -> f32 {
    match raw.source_kind {
        SourceKind::GeoTiff => (raw.spacing[0].abs() * METERS_PER_DEGREE).max(1.0),
        SourceKind::Dicom => raw.spacing[0].abs().max(1e-3),
    }
}

pub fn build_from_geotiff(
    raw: &RawVolume,
    density: f32,
    vertical_exaggeration: f32,
    thresholds: &ThresholdConfig,
) -> VoxelGrid {
    let raw_w = raw.dims[0] as usize;
    let raw_h = raw.dims[1] as usize;
    let pixel_spacing_m = estimate_pixel_spacing_m(raw);
    let world_x_m = (raw_w as f32) * pixel_spacing_m;
    let world_z_m = (raw_h as f32) * pixel_spacing_m;
    let elev_min = thresholds.min;
    let elev_max = thresholds.max;
    let elev_range_m = ((elev_max - elev_min) * vertical_exaggeration).max(0.0);

    let dims_x = (world_x_m / density).ceil().max(1.0) as u32;
    let dims_y = (elev_range_m / density).ceil().max(1.0) as u32;
    let dims_z = (world_z_m / density).ceil().max(1.0) as u32;
    let dims = [dims_x, dims_y, dims_z];

    let row_stride = dims[0] as usize;
    let slice_stride = row_stride * (dims[1] as usize);
    let total = slice_stride * (dims[2] as usize);
    let mut data = vec![0u8; total];

    for vz in 0..dims[2] as usize {
        for vx in 0..dims[0] as usize {
            let raw_x = ((vx as f32 + 0.5) * density / pixel_spacing_m).floor() as usize;
            let raw_z = ((vz as f32 + 0.5) * density / pixel_spacing_m).floor() as usize;
            let raw_x = raw_x.min(raw_w - 1);
            let raw_z = raw_z.min(raw_h - 1);
            let elev = raw.data[raw_z * raw_w + raw_x];
            if !elev.is_finite() {
                continue;
            }
            let scaled = (elev - elev_min) * vertical_exaggeration;
            let y_top = (scaled / density).round().clamp(0.0, dims[1] as f32) as u32;
            for iy in 0..y_top {
                let cell_elev = elev_min + (iy as f32 * density) / vertical_exaggeration.max(1e-6);
                let band = classify_band(cell_elev, elev_min, elev_max);
                let idx = vx + row_stride * (iy as usize) + slice_stride * vz;
                data[idx] = band;
            }
        }
    }

    VoxelGrid {
        data,
        dims,
        density,
        elev_min,
        vertical_exaggeration,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_raw() -> RawVolume {
        // 4x4 raw at ~1 arc-sec → ~30m/pixel
        RawVolume {
            data: vec![
                0.0, 10.0, 20.0, 30.0, 0.0, 10.0, 20.0, 30.0, 0.0, 10.0, 20.0, 30.0, 0.0, 10.0,
                20.0, 30.0,
            ],
            dims: [4, 4, 1],
            spacing: [1.0 / 3600.0, 1.0 / 3600.0, 1.0],
            origin: [0.0, 0.0, 0.0],
            source_kind: SourceKind::GeoTiff,
        }
    }

    #[test]
    fn density_matches_world_extent() {
        let raw = tiny_raw();
        let thresholds = ThresholdConfig {
            min: 0.0,
            max: 30.0,
        };
        // ~30m/pixel × 4 pixels ≈ 124m world extent
        let grid = build_from_geotiff(&raw, 30.0, 1.0, &thresholds);
        assert_eq!(grid.dims[0], 5);
        assert_eq!(grid.dims[2], 5);
    }

    #[test]
    fn larger_density_means_fewer_voxels() {
        let raw = tiny_raw();
        let thresholds = ThresholdConfig {
            min: 0.0,
            max: 30.0,
        };
        let coarse = build_from_geotiff(&raw, 60.0, 1.0, &thresholds);
        let fine = build_from_geotiff(&raw, 30.0, 1.0, &thresholds);
        assert!(coarse.dims[0] < fine.dims[0]);
        assert!(coarse.dims[2] < fine.dims[2]);
    }

    #[test]
    fn nan_column_stays_empty() {
        let mut raw = tiny_raw();
        raw.data[0] = f32::NAN;
        let thresholds = ThresholdConfig {
            min: 0.0,
            max: 30.0,
        };
        let grid = build_from_geotiff(&raw, 30.0, 1.0, &thresholds);
        for iy in 0..grid.dims[1] {
            assert_eq!(grid.get(0, iy, 0), 0);
        }
    }

    #[test]
    fn vertical_exaggeration_scales_height() {
        let raw = tiny_raw();
        let thresholds = ThresholdConfig {
            min: 0.0,
            max: 30.0,
        };
        let g1 = build_from_geotiff(&raw, 30.0, 1.0, &thresholds);
        let g2 = build_from_geotiff(&raw, 30.0, 2.0, &thresholds);
        assert_eq!(g2.dims[1], g1.dims[1] * 2);
    }
}
