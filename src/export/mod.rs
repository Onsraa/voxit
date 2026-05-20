use bevy::prelude::*;

use crate::state::AppState;

pub mod components;
pub mod constants;
pub mod resources;
pub mod systems;
pub mod writer;

#[derive(Event, Debug)]
pub struct ExportRequested;

pub struct VoxFileSaver;

pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExportRequested>()
            .add_systems(
                Update,
                systems::handle_export_request.run_if(in_state(AppState::Previewing)),
            )
            .add_systems(
                Update,
                (systems::handle_save_complete, systems::handle_save_canceled)
                    .run_if(in_state(AppState::Exporting)),
            );
    }
}
