use bevy::prelude::*;
use bitvec::prelude::*;

use crate::ui::resources::PreviewSettings;
use crate::volume::VoxelGrid;

#[derive(Resource)]
pub struct VisibilityMask {
    bits: BitVec,
    dims: [u32; 3],
    visible_count: u32,
}

impl VisibilityMask {
    pub fn compute(grid: &VoxelGrid, settings: &PreviewSettings) -> Self {
        let dx = grid.dims[0] as usize;
        let dy = grid.dims[1] as usize;
        let dz = grid.dims[2] as usize;
        let mut bits = BitVec::with_capacity(dx * dy * dz);
        let mut visible_count = 0u32;

        let cx0 = (settings.crop_x[0].clamp(0.0, 1.0) * dx as f32).floor() as usize;
        let cx1 = (settings.crop_x[1].clamp(0.0, 1.0) * dx as f32).ceil() as usize;
        let cy0 = (settings.crop_y[0].clamp(0.0, 1.0) * dy as f32).floor() as usize;
        let cy1 = (settings.crop_y[1].clamp(0.0, 1.0) * dy as f32).ceil() as usize;
        let cz0 = (settings.crop_z[0].clamp(0.0, 1.0) * dz as f32).floor() as usize;
        let cz1 = (settings.crop_z[1].clamp(0.0, 1.0) * dz as f32).ceil() as usize;
        let cx1 = cx1.min(dx);
        let cy1 = cy1.min(dy);
        let cz1 = cz1.min(dz);

        for z in 0..dz {
            for y in 0..dy {
                for x in 0..dx {
                    let cell = grid.data[x + dx * y + dx * dy * z];
                    let in_crop =
                        x >= cx0 && x < cx1 && y >= cy0 && y < cy1 && z >= cz0 && z < cz1;
                    let cell_elev = grid.elev_min + (y as f32) * grid.density;
                    let elev_ok = cell_elev >= settings.threshold_min
                        && cell_elev <= settings.threshold_max
                        && cell_elev >= settings.sea_level_m;
                    let visible = cell != 0 && in_crop && elev_ok;
                    bits.push(visible);
                    if visible {
                        visible_count += 1;
                    }
                }
            }
        }

        Self {
            bits,
            dims: grid.dims,
            visible_count,
        }
    }

    pub fn visible_count(&self) -> u32 {
        self.visible_count
    }

    pub fn is_visible(&self, x: i32, y: i32, z: i32) -> bool {
        if x < 0 || y < 0 || z < 0 {
            return false;
        }
        let dx = self.dims[0] as i32;
        let dy = self.dims[1] as i32;
        let dz = self.dims[2] as i32;
        if x >= dx || y >= dy || z >= dz {
            return false;
        }
        let idx = (x as usize)
            + (self.dims[0] as usize) * (y as usize)
            + (self.dims[0] as usize) * (self.dims[1] as usize) * (z as usize);
        *self.bits.get(idx).unwrap()
    }
}
