use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_file_dialog::prelude::*;

use voxit::export::{ExportPlugin, VoxFileSaver};
use voxit::render::RenderPlugin;
use voxit::source::SourcePlugin;
use voxit::state::AppState;
use voxit::ui::load_screen::VolumeFilePicker;
use voxit::ui::UiPlugin;

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
