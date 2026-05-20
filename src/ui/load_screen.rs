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
use super::resources::LastLoadError;

pub struct VolumeFilePicker;

pub fn idle_screen(
    mut contexts: EguiContexts,
    mut commands: Commands,
    last_error: Res<LastLoadError>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(CARD_BG))
        .show(ctx, |ui| {
            ui.add_space(CARD_PADDING_TOP);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("voxit")
                        .color(CARD_HEADING)
                        .size(HEADING_SIZE),
                );
                ui.add_space(VERTICAL_GAP_SMALL);
                ui.label(
                    egui::RichText::new("Drop a .tif, .obj, or .glb file on this window")
                        .color(CARD_SUBTEXT)
                        .size(SUBTEXT_SIZE),
                );
                ui.add_space(VERTICAL_GAP_BIG);
                let file_btn = ui.add(
                    egui::Button::new(egui::RichText::new("Open File…").size(BUTTON_TEXT_SIZE))
                        .min_size(egui::vec2(180.0, 40.0)),
                );
                if file_btn.clicked() {
                    commands
                        .dialog()
                        .add_filter("GeoTIFF", &["tif", "tiff"])
                        .add_filter("Mesh (OBJ / glTF)", &["obj", "glb", "gltf"])
                        .pick_file_path::<VolumeFilePicker>();
                }
                if let Some(msg) = &last_error.message {
                    ui.add_space(VERTICAL_GAP_BIG);
                    ui.label(
                        egui::RichText::new(format!("Last load failed: {}", msg))
                            .color(egui::Color32::from_rgb(255, 110, 110))
                            .size(SUBTEXT_SIZE),
                    );
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
