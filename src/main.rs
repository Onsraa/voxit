use bevy::prelude::*;

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
        .run();
}
