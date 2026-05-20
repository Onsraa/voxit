use bevy::prelude::*;
use bevy::tasks::futures_lite::future;

use crate::state::AppState;
use crate::tasks::spawn_async;
use crate::ui::resources::LastLoadError;

use super::components::ParseTask;
use super::geotiff::GeoTiffSource;
use super::{LoadRequested, RawVolume, VolumeSource};

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

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase());
    match ext.as_deref() {
        Some("tif") | Some("tiff") => {}
        _ => {
            let msg = format!("unsupported file (only .tif / .tiff): {}", path.display());
            warn!("{}", msg);
            last_error.message = Some(msg);
            return;
        }
    }

    info!("loading GeoTIFF from {}", path.display());
    last_error.message = None;
    next_state.set(AppState::Loading);
    let task = spawn_async(move || GeoTiffSource::parse(&path));
    commands.spawn(ParseTask(task));
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
            Ok(volume) => {
                log_stats(&volume);
                last_error.message = None;
                commands.insert_resource(volume);
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

fn log_stats(v: &RawVolume) {
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
