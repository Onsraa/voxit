use bevy::prelude::*;
use bevy::window::FileDragAndDrop;
use bevy_egui::{egui, EguiContexts};
use bevy_file_dialog::prelude::*;

use crate::source::LoadRequested;
use crate::state::AppState;

use super::constants::{
    BUTTON_TEXT_SIZE, CARD_BG, CARD_HEADING, CARD_PADDING_TOP, CARD_SUBTEXT, HEADING_SIZE,
    SUBTEXT_SIZE, VERTICAL_GAP_BIG, VERTICAL_GAP_SMALL,
};

pub struct VolumeFilePicker;

pub fn idle_screen(mut contexts: EguiContexts, mut commands: Commands) {
    let ctx = contexts.ctx_mut();
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(CARD_BG))
        .show(ctx, |ui| {
            ui.add_space(CARD_PADDING_TOP);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("voxel-viewer")
                        .color(CARD_HEADING)
                        .size(HEADING_SIZE),
                );
                ui.add_space(VERTICAL_GAP_SMALL);
                ui.label(
                    egui::RichText::new("Drop a .tif file anywhere on this window")
                        .color(CARD_SUBTEXT)
                        .size(SUBTEXT_SIZE),
                );
                ui.add_space(VERTICAL_GAP_BIG);
                let btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new("Open File…").size(BUTTON_TEXT_SIZE),
                    )
                    .min_size(egui::vec2(180.0, 40.0)),
                );
                if btn.clicked() {
                    commands
                        .dialog()
                        .add_filter("GeoTIFF", &["tif", "tiff"])
                        .pick_file_path::<VolumeFilePicker>();
                }
            });
        });
}

pub fn handle_drag_drop(
    state: Res<State<AppState>>,
    mut events: EventReader<FileDragAndDrop>,
    mut load_events: EventWriter<LoadRequested>,
) {
    if *state.get() != AppState::Idle {
        events.clear();
        return;
    }
    for ev in events.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = ev {
            load_events.send(LoadRequested {
                path: path_buf.clone(),
            });
        }
    }
}

pub fn handle_dialog_pick(
    state: Res<State<AppState>>,
    mut events: EventReader<DialogFilePicked<VolumeFilePicker>>,
    mut load_events: EventWriter<LoadRequested>,
) {
    if *state.get() != AppState::Idle {
        events.clear();
        return;
    }
    for ev in events.read() {
        load_events.send(LoadRequested {
            path: ev.path.clone(),
        });
    }
}
