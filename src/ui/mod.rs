use bevy::prelude::*;

use crate::state::AppState;

pub mod components;
pub mod constants;
pub mod load_screen;
pub mod progress;
pub mod resources;
pub mod systems;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
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
        );
    }
}
