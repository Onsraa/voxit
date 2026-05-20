use bevy::prelude::*;
use rayon::prelude::*;

use crate::classify::{classify_band, BAND_COUNT};
use crate::source::mesh::MeshData;
use crate::source::{RawVolume, ThresholdConfig};

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

// Approx meters per raw pixel along the longest horizontal axis. Phase 5
// assumes EPSG:4326 (degrees) — spacing × ~111 km/deg. Projected CRS support
// is post-v1 work.
const METERS_PER_DEGREE: f32 = 111_319.0;

pub fn estimate_pixel_spacing_m(raw: &RawVolume) -> f32 {
    (raw.spacing[0].abs() * METERS_PER_DEGREE).max(1.0)
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

    // Parallelize by Z-slab. Each slab fills independently; no inter-slab
    // dependencies because grid layout is x + dx*y + dx*dy*z.
    data.par_chunks_mut(slice_stride)
        .enumerate()
        .for_each(|(vz, slab)| {
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
                    let cell_elev =
                        elev_min + (iy as f32 * density) / vertical_exaggeration.max(1e-6);
                    let band = classify_band(cell_elev, elev_min, elev_max);
                    let idx = vx + row_stride * (iy as usize);
                    slab[idx] = band;
                }
            }
        });

    VoxelGrid {
        data,
        dims,
        density,
        elev_min,
        vertical_exaggeration,
    }
}

/// How to color voxels produced from a mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshColorMode {
    /// Vertex colors → texture → uniform white. No terrain palette unless
    /// asked.
    Auto,
    /// Force per-triangle average color from vertex colors (no fallback).
    VertexOnly,
    /// Force texture sampling at the triangle centroid UV.
    TextureOnly,
    /// Y-position banded against the terrain biome palette.
    HeightBanded,
    /// Every voxel gets the same near-white band (palette slot 8).
    UniformWhite,
}

pub fn build_from_mesh(
    mesh: &MeshData,
    voxel_size: f32,
    color_mode: MeshColorMode,
    yaw_quarters: u32,
    pitch_quarters: u32,
) -> VoxelGrid {
    let yaw = yaw_quarters % 4;
    let pitch = pitch_quarters % 4;
    let rotated_vertices: Vec<[f32; 3]> = mesh
        .vertices
        .iter()
        .map(|v| {
            let v = rotate_y_quarter(*v, yaw);
            rotate_x_quarter(v, pitch)
        })
        .collect();
    let (aabb_min, aabb_max) = compute_aabb(&rotated_vertices);

    let extent = [
        (aabb_max[0] - aabb_min[0]).max(voxel_size),
        (aabb_max[1] - aabb_min[1]).max(voxel_size),
        (aabb_max[2] - aabb_min[2]).max(voxel_size),
    ];
    let dims = [
        (extent[0] / voxel_size).ceil().max(1.0) as u32,
        (extent[1] / voxel_size).ceil().max(1.0) as u32,
        (extent[2] / voxel_size).ceil().max(1.0) as u32,
    ];
    let total = (dims[0] as usize) * (dims[1] as usize) * (dims[2] as usize);
    let mut data = vec![0u8; total];

    let inv = 1.0 / voxel_size;
    let origin = aabb_min;
    let triangle_count = mesh.indices.len() / 3;

    let writes: Vec<(usize, u8)> = (0..triangle_count)
        .into_par_iter()
        .fold(Vec::new, |mut acc, tri_i| {
            rasterize_triangle(
                mesh,
                &rotated_vertices,
                tri_i,
                origin,
                inv,
                dims,
                color_mode,
                &mut acc,
            );
            acc
        })
        .reduce(Vec::new, |mut a, b| {
            a.extend(b);
            a
        });

    for (idx, band) in writes {
        if idx < data.len() {
            data[idx] = band;
        }
    }

    VoxelGrid {
        data,
        dims,
        density: voxel_size,
        elev_min: aabb_min[1],
        vertical_exaggeration: 1.0,
    }
}

fn rotate_y_quarter(v: [f32; 3], quarters: u32) -> [f32; 3] {
    match quarters % 4 {
        0 => v,
        1 => [v[2], v[1], -v[0]],
        2 => [-v[0], v[1], -v[2]],
        3 => [-v[2], v[1], v[0]],
        _ => unreachable!(),
    }
}

fn rotate_x_quarter(v: [f32; 3], quarters: u32) -> [f32; 3] {
    match quarters % 4 {
        0 => v,
        1 => [v[0], -v[2], v[1]],
        2 => [v[0], -v[1], -v[2]],
        3 => [v[0], v[2], -v[1]],
        _ => unreachable!(),
    }
}

fn compute_aabb(verts: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut aabb_min = [f32::INFINITY; 3];
    let mut aabb_max = [f32::NEG_INFINITY; 3];
    for v in verts {
        for axis in 0..3 {
            if v[axis] < aabb_min[axis] {
                aabb_min[axis] = v[axis];
            }
            if v[axis] > aabb_max[axis] {
                aabb_max[axis] = v[axis];
            }
        }
    }
    (aabb_min, aabb_max)
}

fn rasterize_triangle(
    mesh: &MeshData,
    rotated_vertices: &[[f32; 3]],
    tri_i: usize,
    origin: [f32; 3],
    inv_voxel_size: f32,
    dims: [u32; 3],
    color_mode: MeshColorMode,
    out: &mut Vec<(usize, u8)>,
) {
    let i0 = mesh.indices[tri_i * 3] as usize;
    let i1 = mesh.indices[tri_i * 3 + 1] as usize;
    let i2 = mesh.indices[tri_i * 3 + 2] as usize;
    let v0 = world_to_voxel(rotated_vertices[i0], origin, inv_voxel_size);
    let v1 = world_to_voxel(rotated_vertices[i1], origin, inv_voxel_size);
    let v2 = world_to_voxel(rotated_vertices[i2], origin, inv_voxel_size);

    let band = triangle_band(mesh, [i0, i1, i2], v0, v1, v2, dims[1], color_mode);
    if band == 0 {
        return;
    }

    // Pick sample density so adjacent samples are ~half a voxel apart on the
    // longest edge of the triangle in voxel space.
    let e0 = edge_len(v0, v1);
    let e1 = edge_len(v1, v2);
    let e2 = edge_len(v2, v0);
    let longest = e0.max(e1).max(e2);
    let samples = (longest * 2.0).ceil().max(2.0) as u32;

    let dx = dims[0];
    let dy = dims[1];
    let dz = dims[2];

    for i in 0..=samples {
        let u = i as f32 / samples as f32;
        for j in 0..=(samples - i) {
            let v = j as f32 / samples as f32;
            let w = 1.0 - u - v;
            let p = [
                v0[0] * w + v1[0] * u + v2[0] * v,
                v0[1] * w + v1[1] * u + v2[1] * v,
                v0[2] * w + v1[2] * u + v2[2] * v,
            ];
            let vx = p[0] as i32;
            let vy = p[1] as i32;
            let vz = p[2] as i32;
            if vx < 0 || vy < 0 || vz < 0 {
                continue;
            }
            let (vxu, vyu, vzu) = (vx as u32, vy as u32, vz as u32);
            if vxu >= dx || vyu >= dy || vzu >= dz {
                continue;
            }
            let idx = (vxu as usize)
                + (dx as usize) * (vyu as usize)
                + (dx as usize * dy as usize) * (vzu as usize);
            out.push((idx, band));
        }
    }
}

fn world_to_voxel(p: [f32; 3], origin: [f32; 3], inv: f32) -> [f32; 3] {
    [
        (p[0] - origin[0]) * inv,
        (p[1] - origin[1]) * inv,
        (p[2] - origin[2]) * inv,
    ]
}

fn edge_len(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn triangle_band(
    mesh: &MeshData,
    idx: [usize; 3],
    v0_voxel: [f32; 3],
    v1_voxel: [f32; 3],
    v2_voxel: [f32; 3],
    dim_y: u32,
    mode: MeshColorMode,
) -> u8 {
    match mode {
        MeshColorMode::VertexOnly => band_from_vertex_colors(mesh, idx).unwrap_or(0),
        MeshColorMode::TextureOnly => band_from_texture(mesh, idx).unwrap_or(0),
        MeshColorMode::HeightBanded => {
            band_from_height(v0_voxel[1], v1_voxel[1], v2_voxel[1], dim_y)
        }
        MeshColorMode::UniformWhite => BAND_COUNT, // palette slot 8 = near-white
        MeshColorMode::Auto => band_from_vertex_colors(mesh, idx)
            .or_else(|| band_from_texture(mesh, idx))
            .unwrap_or(BAND_COUNT),
    }
}

fn band_from_height(y0: f32, y1: f32, y2: f32, dim_y: u32) -> u8 {
    let avg = (y0 + y1 + y2) / 3.0;
    let norm = (avg / dim_y.max(1) as f32).clamp(0.0, 1.0);
    let band = (norm * BAND_COUNT as f32).floor() as u8;
    band.min(BAND_COUNT - 1) + 1
}

fn band_from_vertex_colors(mesh: &MeshData, idx: [usize; 3]) -> Option<u8> {
    let colors = mesh.colors.as_ref()?;
    let c0 = colors.get(idx[0])?;
    let c1 = colors.get(idx[1])?;
    let c2 = colors.get(idx[2])?;
    let r = (c0[0] + c1[0] + c2[0]) / 3.0;
    let g = (c0[1] + c1[1] + c2[1]) / 3.0;
    let b = (c0[2] + c1[2] + c2[2]) / 3.0;
    Some(rgb_to_band(r, g, b))
}

fn band_from_texture(mesh: &MeshData, idx: [usize; 3]) -> Option<u8> {
    let tex = mesh.texture.as_ref()?;
    let uvs = mesh.uvs.as_ref()?;
    let u0 = uvs.get(idx[0])?;
    let u1 = uvs.get(idx[1])?;
    let u2 = uvs.get(idx[2])?;
    let uc = (u0[0] + u1[0] + u2[0]) / 3.0;
    let vc = (u0[1] + u1[1] + u2[1]) / 3.0;
    // Wrap UVs
    let uc = uc.rem_euclid(1.0);
    let vc = vc.rem_euclid(1.0);
    let tx = (uc * tex.width as f32) as u32;
    let ty = (vc * tex.height as f32) as u32;
    let tx = tx.min(tex.width.saturating_sub(1));
    let ty = ty.min(tex.height.saturating_sub(1));
    let off = ((ty * tex.width + tx) * 4) as usize;
    let r = tex.rgba8.get(off).copied()? as f32 / 255.0;
    let g = tex.rgba8.get(off + 1).copied()? as f32 / 255.0;
    let b = tex.rgba8.get(off + 2).copied()? as f32 / 255.0;
    Some(rgb_to_band(r, g, b))
}

fn rgb_to_band(r: f32, g: f32, b: f32) -> u8 {
    // Map an arbitrary RGB to one of 8 reference palette bands by nearest
    // distance. Reference palette mirrors source::constants::BIOME_PALETTE
    // but in linear-ish space for quick distance compare.
    const REF: [[f32; 3]; 8] = [
        [30.0 / 255.0, 80.0 / 255.0, 160.0 / 255.0],
        [80.0 / 255.0, 130.0 / 255.0, 200.0 / 255.0],
        [220.0 / 255.0, 200.0 / 255.0, 140.0 / 255.0],
        [60.0 / 255.0, 140.0 / 255.0, 60.0 / 255.0],
        [40.0 / 255.0, 100.0 / 255.0, 40.0 / 255.0],
        [120.0 / 255.0, 100.0 / 255.0, 60.0 / 255.0],
        [180.0 / 255.0, 170.0 / 255.0, 160.0 / 255.0],
        [240.0 / 255.0, 240.0 / 255.0, 250.0 / 255.0],
    ];
    let mut best = 0u8;
    let mut best_d = f32::INFINITY;
    for (i, &c) in REF.iter().enumerate() {
        let dr = r - c[0];
        let dg = g - c[1];
        let db = b - c[2];
        let d = dr * dr + dg * dg + db * db;
        if d < best_d {
            best_d = d;
            best = i as u8;
        }
    }
    best + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_raw() -> RawVolume {
        RawVolume {
            data: vec![
                0.0, 10.0, 20.0, 30.0, 0.0, 10.0, 20.0, 30.0, 0.0, 10.0, 20.0, 30.0, 0.0, 10.0,
                20.0, 30.0,
            ],
            dims: [4, 4, 1],
            spacing: [1.0 / 3600.0, 1.0 / 3600.0, 1.0],
            origin: [0.0, 0.0, 0.0],
        }
    }

    #[test]
    fn density_matches_world_extent() {
        let raw = tiny_raw();
        let thresholds = ThresholdConfig {
            min: 0.0,
            max: 30.0,
        };
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
