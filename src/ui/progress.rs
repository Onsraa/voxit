use bevy_egui::{egui, EguiContexts};

use super::constants::{
    CARD_BG, CARD_HEADING, CARD_PADDING_TOP, HEADING_SIZE, VERTICAL_GAP_BIG,
};

pub fn loading_screen(mut contexts: EguiContexts) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    ctx.request_repaint();
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(CARD_BG))
        .show(ctx, |ui| {
            ui.add_space(CARD_PADDING_TOP);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Loading…")
                        .color(CARD_HEADING)
                        .size(HEADING_SIZE),
                );
                ui.add_space(VERTICAL_GAP_BIG);
                ui.spinner();
            });
        });
}
