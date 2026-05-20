use std::path::{Path, PathBuf};

use anyhow::Result;
use bevy::prelude::*;

use crate::state::AppState;

pub mod components;
pub mod constants;
pub mod geotiff;
pub mod mesh;
pub mod resources;
pub mod systems;

use self::mesh::MeshData;

#[derive(Debug)]
pub struct RawVolume {
    pub data: Vec<f32>,
    pub dims: [u32; 3],
    pub spacing: [f32; 3],
    pub origin: [f32; 3],
}

/// The thing that landed from a parser. Lives in the world as a Resource and
/// the render pipeline branches on the variant.
#[derive(Resource, Debug)]
pub enum SourceData {
    Heightmap(RawVolume),
    Mesh(MeshData),
}

impl SourceData {
    pub fn kind(&self) -> SourceKind {
        match self {
            SourceData::Heightmap(_) => SourceKind::Heightmap,
            SourceData::Mesh(_) => SourceKind::Mesh,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Heightmap,
    Mesh,
}

#[derive(Debug, Clone, Copy)]
pub struct ThresholdConfig {
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone)]
pub struct Palette {
    pub name: &'static str,
    pub colors: Vec<[u8; 4]>,
}

pub trait VolumeSource {
    fn parse(path: &Path) -> Result<RawVolume>;
    fn default_thresholds(volume: &RawVolume) -> ThresholdConfig;
    fn palette_preset() -> Palette;
}

#[derive(Event, Debug, Clone)]
pub struct LoadRequested {
    pub path: PathBuf,
}

pub struct SourcePlugin;

impl Plugin for SourcePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadRequested>().add_systems(
            Update,
            (
                systems::handle_load_requests.run_if(in_state(AppState::Idle)),
                systems::poll_parse_task.run_if(in_state(AppState::Loading)),
            ),
        );
    }
}
