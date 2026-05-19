use bevy::prelude::*;

use crate::classify::classify_band;
use crate::source::{RawVolume, ThresholdConfig};

#[derive(Resource, Debug)]
pub struct VoxelGrid {
    pub data: Vec<u8>,
    pub dims: [u32; 3],
    pub density: f32,
    pub elev_min: f32,
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

pub fn build_from_geotiff(
    raw: &RawVolume,
    density: f32,
    thresholds: &ThresholdConfig,
) -> VoxelGrid {
    let w = raw.dims[0] as usize;
    let h = raw.dims[1] as usize;
    let elev_min = thresholds.min;
    let elev_max = thresholds.max;
    let y_voxels = (((elev_max - elev_min) / density).ceil()).max(1.0) as u32;

    let dims = [raw.dims[0], y_voxels, raw.dims[1]];
    let total =
        (dims[0] as usize) * (dims[1] as usize) * (dims[2] as usize);
    let mut data = vec![0u8; total];

    let row_stride = dims[0] as usize;
    let slice_stride = row_stride * dims[1] as usize;

    for iz in 0..h {
        for ix in 0..w {
            let elev = raw.data[iz * w + ix];
            if !elev.is_finite() {
                continue;
            }
            let normalized = (elev - elev_min) / density;
            let y_top = normalized.round().clamp(0.0, dims[1] as f32) as u32;
            for iy in 0..y_top {
                let cell_elev = elev_min + (iy as f32) * density;
                let band = classify_band(cell_elev, elev_min, elev_max);
                let idx = ix + row_stride * (iy as usize) + slice_stride * iz;
                data[idx] = band;
            }
        }
    }

    VoxelGrid {
        data,
        dims,
        density,
        elev_min,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::SourceKind;

    fn tiny_raw() -> RawVolume {
        RawVolume {
            data: vec![0.0, 10.0, 20.0, 30.0],
            dims: [2, 2, 1],
            spacing: [1.0, 1.0, 1.0],
            origin: [0.0, 0.0, 0.0],
            source_kind: SourceKind::GeoTiff,
        }
    }

    #[test]
    fn build_grid_has_expected_dims() {
        let raw = tiny_raw();
        let thresholds = ThresholdConfig {
            min: 0.0,
            max: 30.0,
        };
        let grid = build_from_geotiff(&raw, 10.0, &thresholds);
        assert_eq!(grid.dims, [2, 3, 2]);
        assert_eq!(grid.data.len(), 12);
    }

    #[test]
    fn tallest_column_fills_to_top() {
        let raw = tiny_raw();
        let thresholds = ThresholdConfig {
            min: 0.0,
            max: 30.0,
        };
        let grid = build_from_geotiff(&raw, 10.0, &thresholds);
        // (ix=1, iz=1) → elev 30 → fills all 3 y-voxels
        assert_ne!(grid.get(1, 0, 1), 0);
        assert_ne!(grid.get(1, 1, 1), 0);
        assert_ne!(grid.get(1, 2, 1), 0);
    }

    #[test]
    fn nan_column_stays_empty() {
        let mut raw = tiny_raw();
        raw.data[0] = f32::NAN;
        let thresholds = ThresholdConfig {
            min: 0.0,
            max: 30.0,
        };
        let grid = build_from_geotiff(&raw, 10.0, &thresholds);
        for iy in 0..grid.dims[1] {
            assert_eq!(grid.get(0, iy, 0), 0);
        }
    }
}
