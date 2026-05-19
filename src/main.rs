use bevy::prelude::*;

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
        .add_plugins((UiPlugin, SourcePlugin, RenderPlugin))
        .run();
}
