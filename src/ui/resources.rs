use bevy::prelude::*;

use crate::volume::MeshColorMode;

#[derive(Resource, Debug, Clone)]
pub struct PreviewSettings {
    pub density_m_per_voxel: f32,
    pub threshold_min: f32,
    pub threshold_max: f32,
    pub elev_full_min: f32,
    pub elev_full_max: f32,
    pub crop_x: [f32; 2],
    pub crop_y: [f32; 2],
    pub crop_z: [f32; 2],
    pub grid_dims: [u32; 3],
    pub sea_level_m: f32,
    pub vertical_exaggeration: f32,
    // Mesh-specific fields (ignored for heightmap source).
    pub mesh_voxels_per_axis: u32,
    pub mesh_yaw_quarters: u32,
    pub mesh_pitch_quarters: u32,
    pub mesh_color_mode: MeshColorMode,
    pub mesh_longest_axis_m: f32,
}

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct PreviewStats {
    pub visible_voxels: u32,
    pub triangle_count: u32,
}

#[derive(Event, Debug)]
pub struct MeshDirty;

#[derive(Event, Debug)]
pub struct VolumeDirty;

#[derive(Resource, Debug, Default)]
pub struct VolumeDebounce {
    pub timer: Option<Timer>,
}

/// Set when a `MeshDirty` arrived while a mesh-rebuild task was still in
/// flight. The polling system re-fires `MeshDirty` on completion if this is
/// set so the latest slider position always ends up applied.
#[derive(Resource, Debug, Default)]
pub struct MeshRebuildPending(pub bool);

#[derive(Resource, Debug, Default)]
pub struct LastLoadError {
    pub message: Option<String>,
}
