use anyhow::{anyhow, Result};
use dot_vox::{Color, Dict, DotVoxData, Frame, Layer, Model, SceneNode, ShapeModel, Size, Voxel};

use crate::source::constants::BIOME_PALETTE;
use crate::visibility::VisibilityMask;
use crate::volume::VoxelGrid;

use super::chunk;
use super::constants::{DOT_VOX_VERSION, VOX_MAX_DIM};

pub fn build_dot_vox_data(grid: &VoxelGrid, mask: &VisibilityMask) -> Result<DotVoxData> {
    let palette = build_palette();
    let fits_single_model = grid.dims[0] <= VOX_MAX_DIM
        && grid.dims[1] <= VOX_MAX_DIM
        && grid.dims[2] <= VOX_MAX_DIM;

    if fits_single_model {
        Ok(build_single_model(grid, mask, palette))
    } else {
        build_multi_model(grid, mask, palette)
    }
}

pub fn serialize(data: &DotVoxData) -> Result<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    data.write_vox(&mut buffer)
        .map_err(|e| anyhow!("write_vox failed: {e}"))?;
    Ok(buffer)
}

fn build_palette() -> Vec<Color> {
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
    palette
}

fn build_single_model(grid: &VoxelGrid, mask: &VisibilityMask, palette: Vec<Color>) -> DotVoxData {
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
                voxels.push(Voxel {
                    x: gx as u8,
                    y: gz as u8,
                    z: gy as u8,
                    i: band - 1,
                });
            }
        }
    }

    let model = Model {
        size: Size { x: dx, y: dz, z: dy },
        voxels,
    };

    DotVoxData {
        version: DOT_VOX_VERSION,
        index_map: (0u8..=255).collect(),
        models: vec![model],
        palette,
        materials: vec![],
        scenes: vec![],
        layers: vec![],
    }
}

fn build_multi_model(
    grid: &VoxelGrid,
    mask: &VisibilityMask,
    palette: Vec<Color>,
) -> Result<DotVoxData> {
    let chunks = chunk::split(grid, mask);
    if chunks.is_empty() {
        return Err(anyhow!("no visible voxels to export"));
    }

    let mut models: Vec<Model> = Vec::with_capacity(chunks.len());
    for c in &chunks {
        models.push(Model {
            size: Size {
                x: c.size[0],
                y: c.size[2],
                z: c.size[1],
            },
            voxels: c.voxels.clone(),
        });
    }

    // Scene graph:
    //   node 0: root nTRN → child 1
    //   node 1: nGRP → children [2, 4, 6, ...]
    //   node (2 + 2i): nTRN with chunk position → child (3 + 2i)
    //   node (3 + 2i): nSHP referencing model i
    let mut scenes: Vec<SceneNode> = Vec::with_capacity(2 + 2 * chunks.len());

    scenes.push(SceneNode::Transform {
        attributes: Dict::new(),
        frames: vec![Frame::new(Dict::new())],
        child: 1,
        layer_id: 0,
    });

    let group_children: Vec<u32> = (0..chunks.len() as u32).map(|i| 2 + 2 * i).collect();
    scenes.push(SceneNode::Group {
        attributes: Dict::new(),
        children: group_children,
    });

    for (i, c) in chunks.iter().enumerate() {
        // Pivot is at chunk center. Our grid (gx, gy, gz) → .vox (gx, gz, gy).
        let our_center_x = (c.origin[0] + c.size[0] / 2) as i32;
        let our_center_y = (c.origin[1] + c.size[1] / 2) as i32;
        let our_center_z = (c.origin[2] + c.size[2] / 2) as i32;
        let vox_x = our_center_x;
        let vox_y = our_center_z;
        let vox_z = our_center_y;

        let mut frame_attrs: Dict = Dict::new();
        frame_attrs.insert("_t".to_string(), format!("{} {} {}", vox_x, vox_y, vox_z));

        scenes.push(SceneNode::Transform {
            attributes: Dict::new(),
            frames: vec![Frame::new(frame_attrs)],
            child: 3 + 2 * i as u32,
            layer_id: 0,
        });

        scenes.push(SceneNode::Shape {
            attributes: Dict::new(),
            models: vec![ShapeModel {
                model_id: i as u32,
                attributes: Dict::new(),
            }],
        });
    }

    Ok(DotVoxData {
        version: DOT_VOX_VERSION,
        index_map: (0u8..=255).collect(),
        models,
        palette,
        materials: vec![],
        scenes,
        layers: vec![Layer {
            attributes: Dict::new(),
        }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::resources::{BiomeMode, PreviewSettings};

    fn synthetic_grid(dims: [u32; 3]) -> (VoxelGrid, VisibilityMask) {
        let total = (dims[0] * dims[1] * dims[2]) as usize;
        let mut data = vec![0u8; total];
        for z in 0..dims[2] {
            for x in 0..dims[0] {
                let h = (((x + z) as f32 * 0.05).sin().abs() * (dims[1] as f32 * 0.8)) as u32;
                let h = h.min(dims[1]);
                for y in 0..h {
                    let idx = (x + dims[0] * y + dims[0] * dims[1] * z) as usize;
                    data[idx] = ((y / 6) % 8 + 1) as u8;
                }
            }
        }
        let grid = VoxelGrid {
            data,
            dims,
            density: 5.0,
            elev_min: 0.0,
            vertical_exaggeration: 1.0,
        };
        let settings = PreviewSettings {
            density_m_per_voxel: 5.0,
            threshold_min: 0.0,
            threshold_max: 1000.0,
            elev_full_min: 0.0,
            elev_full_max: 1000.0,
            crop_x: [0.0, 1.0],
            crop_y: [0.0, 1.0],
            crop_z: [0.0, 1.0],
            grid_dims: dims,
            sea_level_m: 0.0,
            vertical_exaggeration: 1.0,
            biome_mode: BiomeMode::Elevation,
        };
        let mask = VisibilityMask::compute(&grid, &settings);
        (grid, mask)
    }

    #[test]
    fn single_model_for_small_grid() {
        let (grid, mask) = synthetic_grid([64, 32, 64]);
        let data = build_dot_vox_data(&grid, &mask).expect("build");
        assert_eq!(data.models.len(), 1);
        assert!(data.scenes.is_empty());
    }

    #[test]
    fn multi_model_for_large_grid() {
        let (grid, mask) = synthetic_grid([300, 80, 300]);
        let data = build_dot_vox_data(&grid, &mask).expect("build");
        assert!(
            data.models.len() > 1,
            "expected multi-model, got {}",
            data.models.len()
        );
        // 2 framing nodes (root nTRN + nGRP) + 2 per chunk model (nTRN + nSHP).
        let expected_scene_nodes = 2 + 2 * data.models.len();
        assert_eq!(data.scenes.len(), expected_scene_nodes);
    }

    #[test]
    fn multi_model_roundtrips_through_dot_vox() {
        let (grid, mask) = synthetic_grid([300, 80, 300]);
        let data = build_dot_vox_data(&grid, &mask).expect("build");
        let original_model_count = data.models.len();
        let bytes = serialize(&data).expect("serialize");
        assert!(bytes.starts_with(b"VOX "));
        let parsed = dot_vox::load_bytes(&bytes).expect("parse roundtrip");
        assert_eq!(parsed.models.len(), original_model_count);
        let expected_scene_nodes = 2 + 2 * original_model_count;
        assert_eq!(parsed.scenes.len(), expected_scene_nodes);
    }
}
