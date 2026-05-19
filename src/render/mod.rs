use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

use crate::state::AppState;

pub mod components;
pub mod constants;
pub mod palette;
pub mod resources;
pub mod surface_mesh;
pub mod systems;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .add_systems(Startup, systems::setup_ambient)
            .add_systems(OnEnter(AppState::Previewing), systems::build_preview_on_enter)
            .add_systems(OnExit(AppState::Previewing), systems::teardown_preview);
    }
}
