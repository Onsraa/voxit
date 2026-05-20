use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use rayon::prelude::*;

use crate::visibility::VisibilityMask;
use crate::volume::VoxelGrid;

use super::palette::band_linear;

const FACES: [Face; 6] = [
    Face {
        normal: [1.0, 0.0, 0.0],
        offset: [1, 0, 0],
        corners: [
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
        ],
    },
    Face {
        normal: [-1.0, 0.0, 0.0],
        offset: [-1, 0, 0],
        corners: [
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
    },
    Face {
        normal: [0.0, 1.0, 0.0],
        offset: [0, 1, 0],
        corners: [
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
        ],
    },
    Face {
        normal: [0.0, -1.0, 0.0],
        offset: [0, -1, 0],
        corners: [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    },
    Face {
        normal: [0.0, 0.0, 1.0],
        offset: [0, 0, 1],
        corners: [
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    },
    Face {
        normal: [0.0, 0.0, -1.0],
        offset: [0, 0, -1],
        corners: [
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        ],
    },
];

struct Face {
    normal: [f32; 3],
    offset: [i32; 3],
    corners: [[f32; 3]; 4],
}

#[derive(Default)]
struct SlabBuffers {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

pub fn build_surface_mesh(grid: &VoxelGrid, mask: &VisibilityMask) -> Mesh {
    let dims = grid.dims;
    let slabs: Vec<SlabBuffers> = (0..dims[2] as i32)
        .into_par_iter()
        .map(|z| build_slab(z, grid, mask))
        .collect();

    let total_positions: usize = slabs.iter().map(|s| s.positions.len()).sum();
    let total_indices: usize = slabs.iter().map(|s| s.indices.len()).sum();

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(total_positions);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(total_positions);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(total_positions);
    let mut indices: Vec<u32> = Vec::with_capacity(total_indices);

    for slab in slabs {
        let base = positions.len() as u32;
        positions.extend(slab.positions);
        normals.extend(slab.normals);
        colors.extend(slab.colors);
        indices.extend(slab.indices.into_iter().map(|i| i + base));
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn build_slab(z: i32, grid: &VoxelGrid, mask: &VisibilityMask) -> SlabBuffers {
    let dims = grid.dims;
    let mut buf = SlabBuffers::default();

    for y in 0..dims[1] {
        for x in 0..dims[0] {
            if !mask.is_visible(x as i32, y as i32, z) {
                continue;
            }
            let band = grid.get(x, y, z as u32);
            let color = band_linear(band);
            let origin = [x as f32, y as f32, z as f32];

            for face in &FACES {
                let nx = x as i32 + face.offset[0];
                let ny = y as i32 + face.offset[1];
                let nz = z + face.offset[2];
                if mask.is_visible(nx, ny, nz) {
                    continue;
                }

                let base = buf.positions.len() as u32;
                for corner in &face.corners {
                    buf.positions.push([
                        origin[0] + corner[0],
                        origin[1] + corner[1],
                        origin[2] + corner[2],
                    ]);
                    buf.normals.push(face.normal);
                    buf.colors.push(color);
                }
                buf.indices
                    .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }
    }

    buf
}
