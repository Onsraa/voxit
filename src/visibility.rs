use bevy::prelude::*;
use bitvec::prelude::*;
use rayon::prelude::*;

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
        let slab = dx * dy;
        let total = slab * dz;

        let cx0 = (settings.crop_x[0].clamp(0.0, 1.0) * dx as f32).floor() as usize;
        let cx1 = ((settings.crop_x[1].clamp(0.0, 1.0) * dx as f32).ceil() as usize).min(dx);
        let cy0 = (settings.crop_y[0].clamp(0.0, 1.0) * dy as f32).floor() as usize;
        let cy1 = ((settings.crop_y[1].clamp(0.0, 1.0) * dy as f32).ceil() as usize).min(dy);
        let cz0 = (settings.crop_z[0].clamp(0.0, 1.0) * dz as f32).floor() as usize;
        let cz1 = ((settings.crop_z[1].clamp(0.0, 1.0) * dz as f32).ceil() as usize).min(dz);

        let mut flat = vec![false; total];
        let visible_count: u32 = flat
            .par_chunks_mut(slab)
            .enumerate()
            .map(|(z, out)| {
                if z < cz0 || z >= cz1 {
                    return 0u32;
                }
                let src_z_off = z * slab;
                let mut local = 0u32;
                for y in cy0..cy1 {
                    let cell_elev = grid.elev_min + (y as f32) * grid.density;
                    if cell_elev < settings.threshold_min
                        || cell_elev > settings.threshold_max
                        || cell_elev < settings.sea_level_m
                    {
                        continue;
                    }
                    let row_off = y * dx;
                    let src_row = src_z_off + row_off;
                    let dst_row = row_off;
                    for x in cx0..cx1 {
                        let cell = grid.data[src_row + x];
                        if cell != 0 {
                            out[dst_row + x] = true;
                            local += 1;
                        }
                    }
                }
                local
            })
            .sum();

        let mut bits = BitVec::repeat(false, total);
        for (i, &v) in flat.iter().enumerate() {
            if v {
                bits.set(i, true);
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
