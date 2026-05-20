use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use rayon::prelude::*;

use crate::visibility::VisibilityMask;
use crate::volume::VoxelGrid;

use super::palette::band_linear;

#[derive(Clone, Copy)]
struct Quad {
    corners: [[f32; 3]; 4],
    normal: [f32; 3],
    color: [f32; 4],
}

/// Per-worker scratch: an output quad list + the 2D face mask reused across
/// every slice on the worker. The mask grows once to `du * dv` for the axis
/// then keeps capacity for all subsequent slices.
struct Scratch {
    quads: Vec<Quad>,
    mask: Vec<u8>,
}

impl Scratch {
    fn new() -> Self {
        Self {
            quads: Vec::with_capacity(128),
            mask: Vec::new(),
        }
    }
}

pub fn build_surface_mesh(grid: &VoxelGrid, mask: &VisibilityMask) -> Mesh {
    let dims = grid.dims;

    let z_quads: Vec<Quad> = (0..dims[2] as i32)
        .into_par_iter()
        .fold(Scratch::new, |mut scratch, z| {
            greedy_axis_z(z, 1, grid, mask, &mut scratch);
            greedy_axis_z(z, -1, grid, mask, &mut scratch);
            scratch
        })
        .map(|s| s.quads)
        .reduce(Vec::new, concat);

    let y_quads: Vec<Quad> = (0..dims[1] as i32)
        .into_par_iter()
        .fold(Scratch::new, |mut scratch, y| {
            greedy_axis_y(y, 1, grid, mask, &mut scratch);
            greedy_axis_y(y, -1, grid, mask, &mut scratch);
            scratch
        })
        .map(|s| s.quads)
        .reduce(Vec::new, concat);

    let x_quads: Vec<Quad> = (0..dims[0] as i32)
        .into_par_iter()
        .fold(Scratch::new, |mut scratch, x| {
            greedy_axis_x(x, 1, grid, mask, &mut scratch);
            greedy_axis_x(x, -1, grid, mask, &mut scratch);
            scratch
        })
        .map(|s| s.quads)
        .reduce(Vec::new, concat);

    let total = z_quads.len() + y_quads.len() + x_quads.len();
    let mut all_quads: Vec<Quad> = Vec::with_capacity(total);
    all_quads.extend(z_quads);
    all_quads.extend(y_quads);
    all_quads.extend(x_quads);

    quads_to_mesh(all_quads)
}

fn concat(mut a: Vec<Quad>, b: Vec<Quad>) -> Vec<Quad> {
    a.extend(b);
    a
}

fn quads_to_mesh(quads: Vec<Quad>) -> Mesh {
    let n = quads.len();
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n * 4);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n * 4);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(n * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(n * 6);

    for q in &quads {
        let base = positions.len() as u32;
        for c in &q.corners {
            positions.push(*c);
            normals.push(q.normal);
            colors.push(q.color);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
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

fn greedy_axis_z(z: i32, dir: i32, grid: &VoxelGrid, mask: &VisibilityMask, scratch: &mut Scratch) {
    let dims = grid.dims;
    let du = dims[0];
    let dv = dims[1];
    let neighbor_z = z + dir;
    let face_mask = &mut scratch.mask;
    face_mask.clear();
    face_mask.resize((du * dv) as usize, 0);
    for v in 0..dv {
        for u in 0..du {
            if !mask.is_visible(u as i32, v as i32, z) {
                continue;
            }
            if mask.is_visible(u as i32, v as i32, neighbor_z) {
                continue;
            }
            face_mask[(u + v * du) as usize] = grid.get(u, v, z as u32);
        }
    }
    let z_plane = if dir == 1 { (z + 1) as f32 } else { z as f32 };
    let normal = [0.0, 0.0, dir as f32];
    let out = &mut scratch.quads;
    greedy_scan(face_mask, du, dv, |u, v, w, h, band| {
        let corners = if dir == 1 {
            [
                [u as f32, v as f32, z_plane],
                [(u + w) as f32, v as f32, z_plane],
                [(u + w) as f32, (v + h) as f32, z_plane],
                [u as f32, (v + h) as f32, z_plane],
            ]
        } else {
            [
                [u as f32, v as f32, z_plane],
                [u as f32, (v + h) as f32, z_plane],
                [(u + w) as f32, (v + h) as f32, z_plane],
                [(u + w) as f32, v as f32, z_plane],
            ]
        };
        out.push(Quad {
            corners,
            normal,
            color: band_linear(band),
        });
    });
}

fn greedy_axis_y(y: i32, dir: i32, grid: &VoxelGrid, mask: &VisibilityMask, scratch: &mut Scratch) {
    let dims = grid.dims;
    let du = dims[0];
    let dv = dims[2];
    let neighbor_y = y + dir;
    let face_mask = &mut scratch.mask;
    face_mask.clear();
    face_mask.resize((du * dv) as usize, 0);
    for v in 0..dv {
        for u in 0..du {
            if !mask.is_visible(u as i32, y, v as i32) {
                continue;
            }
            if mask.is_visible(u as i32, neighbor_y, v as i32) {
                continue;
            }
            face_mask[(u + v * du) as usize] = grid.get(u, y as u32, v);
        }
    }
    let y_plane = if dir == 1 { (y + 1) as f32 } else { y as f32 };
    let normal = [0.0, dir as f32, 0.0];
    let out = &mut scratch.quads;
    greedy_scan(face_mask, du, dv, |u, v, w, h, band| {
        let corners = if dir == 1 {
            [
                [u as f32, y_plane, v as f32],
                [u as f32, y_plane, (v + h) as f32],
                [(u + w) as f32, y_plane, (v + h) as f32],
                [(u + w) as f32, y_plane, v as f32],
            ]
        } else {
            [
                [u as f32, y_plane, v as f32],
                [(u + w) as f32, y_plane, v as f32],
                [(u + w) as f32, y_plane, (v + h) as f32],
                [u as f32, y_plane, (v + h) as f32],
            ]
        };
        out.push(Quad {
            corners,
            normal,
            color: band_linear(band),
        });
    });
}

fn greedy_axis_x(x: i32, dir: i32, grid: &VoxelGrid, mask: &VisibilityMask, scratch: &mut Scratch) {
    let dims = grid.dims;
    let du = dims[1];
    let dv = dims[2];
    let neighbor_x = x + dir;
    let face_mask = &mut scratch.mask;
    face_mask.clear();
    face_mask.resize((du * dv) as usize, 0);
    for v in 0..dv {
        for u in 0..du {
            if !mask.is_visible(x, u as i32, v as i32) {
                continue;
            }
            if mask.is_visible(neighbor_x, u as i32, v as i32) {
                continue;
            }
            face_mask[(u + v * du) as usize] = grid.get(x as u32, u, v);
        }
    }
    let x_plane = if dir == 1 { (x + 1) as f32 } else { x as f32 };
    let normal = [dir as f32, 0.0, 0.0];
    let out = &mut scratch.quads;
    greedy_scan(face_mask, du, dv, |u, v, w, h, band| {
        let corners = if dir == 1 {
            [
                [x_plane, u as f32, v as f32],
                [x_plane, (u + w) as f32, v as f32],
                [x_plane, (u + w) as f32, (v + h) as f32],
                [x_plane, u as f32, (v + h) as f32],
            ]
        } else {
            [
                [x_plane, u as f32, v as f32],
                [x_plane, u as f32, (v + h) as f32],
                [x_plane, (u + w) as f32, (v + h) as f32],
                [x_plane, (u + w) as f32, v as f32],
            ]
        };
        out.push(Quad {
            corners,
            normal,
            color: band_linear(band),
        });
    });
}

fn greedy_scan<F>(mask: &mut [u8], du: u32, dv: u32, mut emit: F)
where
    F: FnMut(u32, u32, u32, u32, u8),
{
    let du_usize = du as usize;
    for v in 0..dv {
        let mut u = 0u32;
        while u < du {
            let band = mask[u as usize + (v as usize) * du_usize];
            if band == 0 {
                u += 1;
                continue;
            }
            let mut w = 1u32;
            while u + w < du && mask[(u + w) as usize + (v as usize) * du_usize] == band {
                w += 1;
            }
            let mut h = 1u32;
            'grow: loop {
                if v + h >= dv {
                    break;
                }
                let row_off = (v + h) as usize * du_usize;
                for k in 0..w {
                    if mask[(u + k) as usize + row_off] != band {
                        break 'grow;
                    }
                }
                h += 1;
            }
            emit(u, v, w, h, band);
            for kv in 0..h {
                let row_off = (v + kv) as usize * du_usize;
                for ku in 0..w {
                    mask[(u + ku) as usize + row_off] = 0;
                }
            }
            u += w;
        }
    }
}
