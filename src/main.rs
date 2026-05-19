use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_file_dialog::prelude::*;

mod classify;
mod render;
mod source;
mod state;
mod tasks;
mod ui;
mod visibility;
mod volume;

use render::RenderPlugin;
use source::SourcePlugin;
use state::AppState;
use ui::load_screen::VolumeFilePicker;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "voxel-viewer".to_string(),
                resolution: (1280.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_plugins(EguiPlugin)
        .add_plugins(FileDialogPlugin::new().with_pick_file::<VolumeFilePicker>())
        .add_plugins((UiPlugin, SourcePlugin, RenderPlugin))
        .run();
}
