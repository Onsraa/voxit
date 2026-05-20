use anyhow::{anyhow, Result};
use dot_vox::{Color, DotVoxData, Model, Size, Voxel};

use crate::source::constants::BIOME_PALETTE;
use crate::visibility::VisibilityMask;
use crate::volume::VoxelGrid;

use super::constants::{DOT_VOX_VERSION, VOX_MAX_DIM};

pub fn build_dot_vox_data(grid: &VoxelGrid, mask: &VisibilityMask) -> Result<DotVoxData> {
    if grid.dims[0] > VOX_MAX_DIM || grid.dims[1] > VOX_MAX_DIM || grid.dims[2] > VOX_MAX_DIM {
        return Err(anyhow!(
            "grid {:?} exceeds {}^3 single-model cap — increase voxel size or wait for Phase 7 chunking",
            grid.dims,
            VOX_MAX_DIM
        ));
    }

    let dx = grid.dims[0];
    let dy = grid.dims[1];
    let dz = grid.dims[2];

    let mut voxels: Vec<Voxel> = Vec::new();
    for gz in 0..dz {
        for gy in 0..dy {
            for gx in 0..dx {
                if !mask.is_visible(gx as i32, gy as i32, gz as i32) {
                    continue;
                }
                let band = grid.get(gx, gy, gz);
                if band == 0 {
                    continue;
                }
                // Bevy Y-up → .vox Z-up. dot_vox Voxel.i is the 0-based palette
                // slot in memory; the encoder adds +1 for the file format.
                voxels.push(Voxel {
                    x: gx as u8,
                    y: gz as u8,
                    z: gy as u8,
                    i: band - 1,
                });
            }
        }
    }

    let mut palette: Vec<Color> = vec![
        Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        };
        256
    ];
    for (i, &[r, g, b, a]) in BIOME_PALETTE.iter().enumerate() {
        palette[i] = Color { r, g, b, a };
    }

    let model = Model {
        size: Size { x: dx, y: dz, z: dy },
        voxels,
    };

    Ok(DotVoxData {
        version: DOT_VOX_VERSION,
        index_map: (0u8..=255).collect(),
        models: vec![model],
        palette,
        materials: vec![],
        scenes: vec![],
        layers: vec![],
    })
}

pub fn serialize(data: &DotVoxData) -> Result<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    data.write_vox(&mut buffer)
        .map_err(|e| anyhow!("write_vox failed: {e}"))?;
    Ok(buffer)
}
