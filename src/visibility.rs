use bevy::prelude::*;
use bitvec::prelude::*;

use crate::volume::VoxelGrid;

#[derive(Resource)]
pub struct VisibilityMask {
    bits: BitVec,
    dims: [u32; 3],
}

impl VisibilityMask {
    pub fn from_grid(grid: &VoxelGrid) -> Self {
        let mut bits = BitVec::with_capacity(grid.data.len());
        for &b in &grid.data {
            bits.push(b != 0);
        }
        Self {
            bits,
            dims: grid.dims,
        }
    }

    pub fn dims(&self) -> [u32; 3] {
        self.dims
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
