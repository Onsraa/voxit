use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::resources::{PreviewSettings, PreviewStats};

pub fn hud(
    mut contexts: EguiContexts,
    settings: Res<PreviewSettings>,
    stats: Res<PreviewStats>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    egui::Area::new("preview-hud".into())
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(12.0, 12.0))
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 160))
                .inner_margin(egui::Margin::same(8.0))
                .rounding(4.0)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "grid: {} × {} × {}",
                            settings.grid_dims[0], settings.grid_dims[1], settings.grid_dims[2]
                        ))
                        .monospace()
                        .color(egui::Color32::WHITE),
                    );
                    ui.label(
                        egui::RichText::new(format!("visible voxels: {}", stats.visible_voxels))
                            .monospace()
                            .color(egui::Color32::WHITE),
                    );
                    ui.label(
                        egui::RichText::new(format!("triangles: {}", stats.triangle_count))
                            .monospace()
                            .color(egui::Color32::WHITE),
                    );
                });
        });
}
