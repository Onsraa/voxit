use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_file_dialog::prelude::*;

mod classify;
mod export;
mod render;
mod source;
mod state;
mod tasks;
mod ui;
mod visibility;
mod volume;

use export::{ExportPlugin, VoxFileSaver};
use render::RenderPlugin;
use source::SourcePlugin;
use state::AppState;
use ui::load_screen::VolumeFilePicker;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "voxit".to_string(),
                resolution: (1280.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(
            FileDialogPlugin::new()
                .with_pick_file::<VolumeFilePicker>()
                .with_save_file::<VoxFileSaver>(),
        )
        .add_plugins((UiPlugin, SourcePlugin, RenderPlugin, ExportPlugin))
        .run();
}
