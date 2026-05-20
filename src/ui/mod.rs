use bevy::prelude::*;

use crate::state::AppState;

pub mod components;
pub mod constants;
pub mod geotiff_panel;
pub mod hud;
pub mod load_screen;
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
                (geotiff_panel::geotiff_panel, hud::hud).run_if(in_state(AppState::Previewing)),
            );
    }
}
