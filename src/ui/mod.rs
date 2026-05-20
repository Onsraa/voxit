use bevy::prelude::*;

use crate::source::{SourceData, SourceKind};
use crate::state::AppState;

fn source_is_heightmap(source: Option<Res<SourceData>>) -> bool {
    source.map(|s| s.kind() == SourceKind::Heightmap).unwrap_or(false)
}

fn source_is_mesh(source: Option<Res<SourceData>>) -> bool {
    source.map(|s| s.kind() == SourceKind::Mesh).unwrap_or(false)
}

pub mod components;
pub mod constants;
pub mod geotiff_panel;
pub mod hud;
pub mod load_screen;
pub mod mesh_panel;
pub mod progress;
pub mod resources;
pub mod systems;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<resources::MeshDirty>()
            .add_event::<resources::VolumeDirty>()
            .init_resource::<resources::PreviewStats>()
            .init_resource::<resources::VolumeDebounce>()
            .init_resource::<resources::LastLoadError>()
            .add_systems(
                Update,
                (load_screen::handle_drag_drop, load_screen::handle_dialog_pick),
            )
            .add_systems(
                Update,
                load_screen::idle_screen.run_if(in_state(AppState::Idle)),
            )
            .add_systems(
                Update,
                progress::loading_screen.run_if(in_state(AppState::Loading)),
            )
            .add_systems(
                Update,
                (
                    hud::hud,
                    geotiff_panel::geotiff_panel.run_if(source_is_heightmap),
                    mesh_panel::mesh_panel.run_if(source_is_mesh),
                )
                    .run_if(in_state(AppState::Previewing)),
            );
    }
}
