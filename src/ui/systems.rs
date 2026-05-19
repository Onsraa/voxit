use bevy::prelude::*;

use crate::state::AppState;

use super::components::StateLabel;
use super::constants::{
    BACKGROUND_COLOR, STATE_TEXT_COLOR, STATE_TEXT_EXPORTING, STATE_TEXT_FONT_SIZE,
    STATE_TEXT_IDLE, STATE_TEXT_LOADING, STATE_TEXT_PREVIEWING,
};

pub fn spawn_camera_and_label(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BACKGROUND_COLOR.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    STATE_TEXT_IDLE,
                    TextStyle {
                        font_size: STATE_TEXT_FONT_SIZE,
                        color: STATE_TEXT_COLOR,
                        ..default()
                    },
                ),
                StateLabel,
            ));
        });
}

pub fn update_label_text(
    state: Res<State<AppState>>,
    mut query: Query<&mut Text, With<StateLabel>>,
) {
    let new_text = match state.get() {
        AppState::Idle => STATE_TEXT_IDLE,
        AppState::Loading => STATE_TEXT_LOADING,
        AppState::Previewing => STATE_TEXT_PREVIEWING,
        AppState::Exporting => STATE_TEXT_EXPORTING,
    };
    for mut text in &mut query {
        text.sections[0].value = new_text.to_string();
    }
}

pub fn debug_state_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        next_state.set(AppState::Idle);
    }
    if keys.just_pressed(KeyCode::Digit2) {
        next_state.set(AppState::Loading);
    }
    if keys.just_pressed(KeyCode::Digit3) {
        next_state.set(AppState::Previewing);
    }
    if keys.just_pressed(KeyCode::Digit4) {
        next_state.set(AppState::Exporting);
    }
}
