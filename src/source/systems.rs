use std::path::PathBuf;

use bevy::prelude::*;
use bevy::tasks::futures_lite::future;

use crate::state::AppState;
use crate::tasks::spawn_async;

use super::components::ParseTask;
use super::constants::TEST_GEOTIFF_PATH;
use super::geotiff::GeoTiffSource;
use super::{RawVolume, VolumeSource};

pub fn kick_off_test_load(mut commands: Commands, mut next_state: ResMut<NextState<AppState>>) {
    let path = PathBuf::from(TEST_GEOTIFF_PATH);
    if !path.exists() {
        info!(
            "test GeoTIFF not present at {} — staying in Idle. Drop a .tif at that path to load.",
            path.display()
        );
        return;
    }
    info!("loading GeoTIFF from {}", path.display());
    next_state.set(AppState::Loading);
    let task = spawn_async(move || GeoTiffSource::parse(&path));
    commands.spawn(ParseTask(task));
}

pub fn poll_parse_task(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ParseTask)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (entity, mut task) in &mut tasks {
        let Some(result) = future::block_on(future::poll_once(&mut task.0)) else {
            continue;
        };
        match result {
            Ok(volume) => {
                log_stats(&volume);
                commands.insert_resource(volume);
                next_state.set(AppState::Previewing);
            }
            Err(e) => {
                error!("GeoTIFF parse failed: {:#}", e);
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
