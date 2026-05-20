use bevy::prelude::*;
use bevy::tasks::futures_lite::future;

use crate::state::AppState;
use crate::tasks::spawn_async;
use crate::ui::resources::LastLoadError;

use super::components::ParseTask;
use super::geotiff::GeoTiffSource;
use super::mesh::MeshSource;
use super::{LoadRequested, RawVolume, SourceData, VolumeSource};

enum InputKind {
    GeoTiff,
    Mesh,
}

pub fn handle_load_requests(
    mut commands: Commands,
    mut events: EventReader<LoadRequested>,
    mut next_state: ResMut<NextState<AppState>>,
    mut last_error: ResMut<LastLoadError>,
) {
    let Some(ev) = events.read().next() else {
        return;
    };
    let path = ev.path.clone();
    events.clear();

    let kind = classify_input(&path);
    let Some(kind) = kind else {
        let msg = format!(
            "unsupported file (only .tif/.tiff/.obj/.glb/.gltf): {}",
            path.display()
        );
        warn!("{}", msg);
        last_error.message = Some(msg);
        return;
    };

    let label = match kind {
        InputKind::GeoTiff => "GeoTIFF",
        InputKind::Mesh => "mesh",
    };
    info!("loading {} from {}", label, path.display());
    last_error.message = None;
    next_state.set(AppState::Loading);

    let task = spawn_async(move || match kind {
        InputKind::GeoTiff => GeoTiffSource::parse(&path).map(SourceData::Heightmap),
        InputKind::Mesh => MeshSource::parse(&path).map(SourceData::Mesh),
    });
    commands.spawn(ParseTask(task));
}

fn classify_input(path: &std::path::Path) -> Option<InputKind> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());
    match ext.as_deref() {
        Some("tif") | Some("tiff") => Some(InputKind::GeoTiff),
        Some("obj") | Some("glb") | Some("gltf") => Some(InputKind::Mesh),
        _ => None,
    }
}

pub fn poll_parse_task(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ParseTask)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut last_error: ResMut<LastLoadError>,
) {
    for (entity, mut task) in &mut tasks {
        let Some(result) = future::block_on(future::poll_once(&mut task.0)) else {
            continue;
        };
        match result {
            Ok(source) => {
                log_stats(&source);
                last_error.message = None;
                commands.insert_resource(source);
                next_state.set(AppState::Previewing);
            }
            Err(e) => {
                let msg = format!("{:#}", e);
                error!("parse failed: {}", msg);
                last_error.message = Some(msg);
                next_state.set(AppState::Idle);
            }
        }
        commands.entity(entity).despawn();
    }
}

fn log_stats(source: &SourceData) {
    match source {
        SourceData::Heightmap(v) => log_heightmap_stats(v),
        SourceData::Mesh(m) => {
            info!(
                "mesh parsed: vertices={} tris={} aabb_min={:?} aabb_max={:?} texture={}",
                m.vertices.len(),
                m.indices.len() / 3,
                m.aabb_min,
                m.aabb_max,
                m.texture.is_some()
            );
        }
    }
}

fn log_heightmap_stats(v: &RawVolume) {
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    let mut nan = 0usize;
    for &x in &v.data {
        if x.is_nan() {
            nan += 1;
            continue;
        }
        if x < min {
            min = x;
        }
        if x > max {
            max = x;
        }
    }
    info!(
        "GeoTIFF parsed: dims={:?} spacing={:?} origin={:?} min={:.2} max={:.2} nan={}",
        v.dims, v.spacing, v.origin, min, max, nan
    );
}
