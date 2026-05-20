use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::export::ExportRequested;

use super::resources::{MeshDirty, PreviewSettings, VolumeDirty};

const PANEL_WIDTH: f32 = 280.0;

pub fn geotiff_panel(
    mut contexts: EguiContexts,
    mut settings: ResMut<PreviewSettings>,
    mut mesh_events: EventWriter<MeshDirty>,
    mut volume_events: EventWriter<VolumeDirty>,
    mut export_events: EventWriter<ExportRequested>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    egui::SidePanel::right("geotiff-panel")
        .exact_width(PANEL_WIDTH)
        .resizable(false)
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading("GeoTIFF Controls");
            ui.separator();

            let mut volume_dirty = false;
            let mut mesh_dirty = false;

            ui.label("Voxel size (m) — smaller = denser");
            let r = ui.add(
                egui::Slider::new(&mut settings.density_m_per_voxel, 5.0..=400.0)
                    .logarithmic(true),
            );
            if r.changed() {
                volume_dirty = true;
            }

            ui.add_space(6.0);
            ui.label("Vertical exaggeration");
            let r = ui.add(egui::Slider::new(&mut settings.vertical_exaggeration, 0.1..=10.0));
            if r.changed() {
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
