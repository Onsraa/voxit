use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::export::ExportRequested;
use crate::volume::MeshColorMode;

use super::resources::{MeshDirty, PreviewSettings, VolumeDirty};

const PANEL_WIDTH: f32 = 280.0;

pub fn mesh_panel(
    mut contexts: EguiContexts,
    mut settings: ResMut<PreviewSettings>,
    mut mesh_events: EventWriter<MeshDirty>,
    mut volume_events: EventWriter<VolumeDirty>,
    mut export_events: EventWriter<ExportRequested>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    egui::SidePanel::right("mesh-panel")
        .exact_width(PANEL_WIDTH)
        .resizable(false)
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading("Mesh Controls");
            ui.separator();

            let mut volume_dirty = false;
            let mut mesh_dirty = false;

            ui.label("Voxels along longest axis");
            let r = ui.add(
                egui::Slider::new(&mut settings.mesh_voxels_per_axis, 16..=512).logarithmic(true),
            );
            if r.changed() {
                let n = settings.mesh_voxels_per_axis.max(1) as f32;
                settings.density_m_per_voxel = settings.mesh_longest_axis_m / n;
                volume_dirty = true;
            }
            ui.label(
                egui::RichText::new(format!(
                    "voxel size: {:.4} world units",
                    settings.density_m_per_voxel
                ))
                .small()
                .color(egui::Color32::from_rgb(160, 160, 175)),
            );

            ui.add_space(10.0);
            ui.label("Yaw (around vertical Y)");
            ui.horizontal(|ui| {
                let labels = ["N", "E", "S", "W"];
                for (i, label) in labels.iter().enumerate() {
                    let selected = settings.mesh_yaw_quarters == i as u32;
                    if ui.selectable_label(selected, *label).clicked() && !selected {
                        settings.mesh_yaw_quarters = i as u32;
                        volume_dirty = true;
                    }
                }
            });

            ui.add_space(6.0);
            ui.label("Pitch (around X)");
            ui.horizontal(|ui| {
                let labels = ["0\u{00b0}", "90\u{00b0}", "180\u{00b0}", "270\u{00b0}"];
                for (i, label) in labels.iter().enumerate() {
                    let selected = settings.mesh_pitch_quarters == i as u32;
                    if ui.selectable_label(selected, *label).clicked() && !selected {
                        settings.mesh_pitch_quarters = i as u32;
                        volume_dirty = true;
                    }
                }
            });

            ui.add_space(10.0);
            ui.label("Color source");
            let mut mode = settings.mesh_color_mode;
            egui::ComboBox::from_id_source("mesh-color-mode")
                .selected_text(format!("{:?}", mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut mode, MeshColorMode::Auto, "Auto");
                    ui.selectable_value(&mut mode, MeshColorMode::VertexOnly, "Vertex only");
                    ui.selectable_value(&mut mode, MeshColorMode::TextureOnly, "Texture only");
                    ui.selectable_value(&mut mode, MeshColorMode::HeightBanded, "Height-banded");
                    ui.selectable_value(&mut mode, MeshColorMode::UniformWhite, "Uniform white");
                });
            if mode != settings.mesh_color_mode {
                settings.mesh_color_mode = mode;
                volume_dirty = true;
            }

            ui.add_space(10.0);
            ui.label("Crop X (fraction)");
            let r = ui.add(egui::Slider::new(&mut settings.crop_x[0], 0.0..=1.0));
            if r.changed() {
                if settings.crop_x[0] > settings.crop_x[1] {
                    settings.crop_x[0] = settings.crop_x[1];
                }
                mesh_dirty = true;
            }
            let r = ui.add(egui::Slider::new(&mut settings.crop_x[1], 0.0..=1.0));
            if r.changed() {
                if settings.crop_x[1] < settings.crop_x[0] {
                    settings.crop_x[1] = settings.crop_x[0];
                }
                mesh_dirty = true;
            }

            ui.add_space(6.0);
            ui.label("Crop Y (fraction)");
            let r = ui.add(egui::Slider::new(&mut settings.crop_y[0], 0.0..=1.0));
            if r.changed() {
                if settings.crop_y[0] > settings.crop_y[1] {
                    settings.crop_y[0] = settings.crop_y[1];
                }
                mesh_dirty = true;
            }
            let r = ui.add(egui::Slider::new(&mut settings.crop_y[1], 0.0..=1.0));
            if r.changed() {
                if settings.crop_y[1] < settings.crop_y[0] {
                    settings.crop_y[1] = settings.crop_y[0];
                }
                mesh_dirty = true;
            }

            ui.add_space(6.0);
            ui.label("Crop Z (fraction)");
            let r = ui.add(egui::Slider::new(&mut settings.crop_z[0], 0.0..=1.0));
            if r.changed() {
                if settings.crop_z[0] > settings.crop_z[1] {
                    settings.crop_z[0] = settings.crop_z[1];
                }
                mesh_dirty = true;
            }
            let r = ui.add(egui::Slider::new(&mut settings.crop_z[1], 0.0..=1.0));
            if r.changed() {
                if settings.crop_z[1] < settings.crop_z[0] {
                    settings.crop_z[1] = settings.crop_z[0];
                }
                mesh_dirty = true;
            }

            if volume_dirty {
                volume_events.send(VolumeDirty);
            } else if mesh_dirty {
                mesh_events.send(MeshDirty);
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);
            let export_btn = ui.add(
                egui::Button::new(egui::RichText::new("Export .vox").size(16.0))
                    .min_size(egui::vec2(ui.available_width(), 36.0)),
            );
            if export_btn.clicked() {
                export_events.send(ExportRequested);
            }
        });
}
