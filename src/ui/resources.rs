use bevy::prelude::*;

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
    pub biome_mode: BiomeMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BiomeMode {
    #[default]
    Elevation,
    Slope,
    Flat,
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

#[derive(Resource, Debug, Default)]
pub struct LastLoadError {
    pub message: Option<String>,
}
