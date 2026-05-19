use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

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

pub fn build_surface_mesh(grid: &VoxelGrid, mask: &VisibilityMask) -> Mesh {
    let dims = grid.dims;
    let est_voxels = (dims[0] as usize) * (dims[1] as usize) * (dims[2] as usize);

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(est_voxels / 8);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(est_voxels / 8);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(est_voxels / 8);
    let mut indices: Vec<u32> = Vec::with_capacity(est_voxels / 8);

    for z in 0..dims[2] {
        for y in 0..dims[1] {
            for x in 0..dims[0] {
                if !mask.is_visible(x as i32, y as i32, z as i32) {
                    continue;
                }
                let band = grid.get(x, y, z);
                let color = band_linear(band);
                let origin = [x as f32, y as f32, z as f32];

                for face in &FACES {
                    let nx = x as i32 + face.offset[0];
                    let ny = y as i32 + face.offset[1];
                    let nz = z as i32 + face.offset[2];
                    if mask.is_visible(nx, ny, nz) {
                        continue;
                    }

                    let base = positions.len() as u32;
                    for corner in &face.corners {
                        positions.push([
                            origin[0] + corner[0],
                            origin[1] + corner[1],
                            origin[2] + corner[2],
                        ]);
                        normals.push(face.normal);
                        colors.push(color);
                    }
                    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
                }
            }
        }
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
