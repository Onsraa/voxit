use dot_vox::Voxel;

use crate::visibility::VisibilityMask;
use crate::volume::VoxelGrid;

use super::constants::VOX_MAX_DIM;

pub struct ChunkData {
    pub index: [u32; 3],
    pub origin: [u32; 3],
    pub size: [u32; 3],
    pub voxels: Vec<Voxel>,
}

pub fn chunks_layout(dims: [u32; 3]) -> [u32; 3] {
    [
        dims[0].div_ceil(VOX_MAX_DIM).max(1),
        dims[1].div_ceil(VOX_MAX_DIM).max(1),
        dims[2].div_ceil(VOX_MAX_DIM).max(1),
    ]
}

pub fn split(grid: &VoxelGrid, mask: &VisibilityMask) -> Vec<ChunkData> {
    let layout = chunks_layout(grid.dims);
    let mut chunks = Vec::new();

    for cz in 0..layout[2] {
        for cy in 0..layout[1] {
            for cx in 0..layout[0] {
                let origin = [cx * VOX_MAX_DIM, cy * VOX_MAX_DIM, cz * VOX_MAX_DIM];
                let size = [
                    (grid.dims[0] - origin[0]).min(VOX_MAX_DIM),
                    (grid.dims[1] - origin[1]).min(VOX_MAX_DIM),
                    (grid.dims[2] - origin[2]).min(VOX_MAX_DIM),
                ];
                let mut voxels: Vec<Voxel> = Vec::new();
                for lz in 0..size[2] {
                    for ly in 0..size[1] {
                        for lx in 0..size[0] {
                            let gx = origin[0] + lx;
                            let gy = origin[1] + ly;
                            let gz = origin[2] + lz;
                            if !mask.is_visible(gx as i32, gy as i32, gz as i32) {
                                continue;
                            }
                            let band = grid.get(gx, gy, gz);
                            if band == 0 {
                                continue;
                            }
                            voxels.push(Voxel {
                                x: lx as u8,
                                y: lz as u8,
                                z: ly as u8,
                                i: band - 1,
                            });
                        }
                    }
                }
                if !voxels.is_empty() {
                    chunks.push(ChunkData {
                        index: [cx, cy, cz],
                        origin,
                        size,
                        voxels,
                    });
                }
            }
        }
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_small_grid_is_one_chunk() {
        assert_eq!(chunks_layout([10, 10, 10]), [1, 1, 1]);
    }

    #[test]
    fn layout_exact_256_is_one_chunk() {
        assert_eq!(chunks_layout([256, 256, 256]), [1, 1, 1]);
    }

    #[test]
    fn layout_just_past_256_is_two_chunks() {
        assert_eq!(chunks_layout([257, 100, 100]), [2, 1, 1]);
    }

    #[test]
    fn layout_3d_split() {
        assert_eq!(chunks_layout([500, 600, 1000]), [2, 3, 4]);
    }
}
