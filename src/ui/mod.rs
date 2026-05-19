use bevy::prelude::*;

use crate::state::AppState;

pub mod components;
pub mod constants;
pub mod resources;
pub mod systems;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::spawn_camera_and_label).add_systems(
            Update,
            (
                systems::debug_state_keys,
                systems::update_label_text.run_if(state_changed::<AppState>),
            ),
        );
    }
}
