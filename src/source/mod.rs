use std::path::Path;

use anyhow::Result;
use bevy::prelude::*;

use crate::state::AppState;

pub mod components;
pub mod constants;
pub mod geotiff;
pub mod resources;
pub mod systems;

#[derive(Resource, Debug)]
pub struct RawVolume {
    pub data: Vec<f32>,
    pub dims: [u32; 3],
    pub spacing: [f32; 3],
    pub origin: [f32; 3],
    pub source_kind: SourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    GeoTiff,
    Dicom,
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

pub struct SourcePlugin;

impl Plugin for SourcePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::kick_off_test_load)
            .add_systems(
                Update,
                systems::poll_parse_task.run_if(in_state(AppState::Loading)),
            );
    }
}
