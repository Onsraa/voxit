use std::path::Path;

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Option<Vec<[f32; 2]>>,
    pub colors: Option<Vec<[f32; 4]>>,
    pub indices: Vec<u32>,
    pub texture: Option<MeshTexture>,
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct MeshTexture {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

pub struct MeshSource;

impl MeshSource {
    pub fn parse(path: &Path) -> Result<MeshData> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase());
        match ext.as_deref() {
            Some("obj") => parse_obj(path),
            Some("glb") | Some("gltf") => parse_gltf(path),
            other => Err(anyhow!("unsupported mesh extension: {:?}", other)),
        }
    }
}

fn parse_obj(path: &Path) -> Result<MeshData> {
    let (models, _materials) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ignore_lines: true,
            ignore_points: true,
            ..Default::default()
        },
    )
    .with_context(|| format!("load OBJ {}", path.display()))?;

    if models.is_empty() {
        return Err(anyhow!("OBJ contains no meshes"));
    }

    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut any_normals = false;
    let mut any_uvs = false;

    for model in &models {
        let mesh = &model.mesh;
        let base = vertices.len() as u32;

        let positions = &mesh.positions;
        for i in 0..(positions.len() / 3) {
            vertices.push([positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]]);
        }

        if !mesh.normals.is_empty() {
            any_normals = true;
            for i in 0..(mesh.normals.len() / 3) {
                normals.push([
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ]);
            }
        }

        if !mesh.texcoords.is_empty() {
            any_uvs = true;
            for i in 0..(mesh.texcoords.len() / 2) {
                uvs.push([mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]]);
            }
        }

        for idx in &mesh.indices {
            indices.push(base + idx);
        }
    }

    if indices.is_empty() {
        return Err(anyhow!("OBJ has no triangles after parse"));
    }

    let (aabb_min, aabb_max) = compute_aabb(&vertices);

    Ok(MeshData {
        vertices,
        normals: if any_normals { normals } else { Vec::new() },
        uvs: if any_uvs { Some(uvs) } else { None },
        colors: None,
        indices,
        texture: None,
        aabb_min,
        aabb_max,
    })
}

fn parse_gltf(path: &Path) -> Result<MeshData> {
    let (document, buffers, images) =
        gltf::import(path).with_context(|| format!("load glTF {}", path.display()))?;

    let mesh = document
        .meshes()
        .next()
        .ok_or_else(|| anyhow!("glTF has no meshes"))?;
    let primitive = mesh
        .primitives()
        .next()
        .ok_or_else(|| anyhow!("first mesh has no primitives"))?;

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or_else(|| anyhow!("primitive missing POSITION attribute"))?
        .collect();

    let normals: Vec<[f32; 3]> = reader
        .read_normals()
        .map(|iter| iter.collect())
        .unwrap_or_default();

    let uvs: Option<Vec<[f32; 2]>> = reader
        .read_tex_coords(0)
        .map(|tc| tc.into_f32().collect());

    let colors: Option<Vec<[f32; 4]>> = reader
        .read_colors(0)
        .map(|c| c.into_rgba_f32().collect());

    let indices: Vec<u32> = reader
        .read_indices()
        .map(|i| i.into_u32().collect())
        .unwrap_or_else(|| (0..positions.len() as u32).collect());

    if indices.is_empty() || positions.is_empty() {
        return Err(anyhow!("primitive has no triangles or no positions"));
    }

    let texture = primitive
        .material()
        .pbr_metallic_roughness()
        .base_color_texture()
        .and_then(|info| {
            let src = info.texture().source().index();
            convert_image(&images[src])
        });

    let (aabb_min, aabb_max) = compute_aabb(&positions);

    Ok(MeshData {
        vertices: positions,
        normals,
        uvs,
        colors,
        indices,
        texture,
        aabb_min,
        aabb_max,
    })
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

fn convert_image(img: &gltf::image::Data) -> Option<MeshTexture> {
    use gltf::image::Format;
    let rgba8 = match img.format {
        Format::R8G8B8A8 => img.pixels.clone(),
        Format::R8G8B8 => {
            let mut out = Vec::with_capacity(img.pixels.len() * 4 / 3);
            for chunk in img.pixels.chunks_exact(3) {
                out.push(chunk[0]);
                out.push(chunk[1]);
                out.push(chunk[2]);
                out.push(255);
            }
            out
        }
        _ => return None,
    };
    Some(MeshTexture {
        width: img.width,
        height: img.height,
        rgba8,
    })
}
