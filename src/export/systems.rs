use bevy::prelude::*;
use bevy_file_dialog::prelude::*;

use crate::state::AppState;
use crate::visibility::VisibilityMask;
use crate::volume::VoxelGrid;

use super::constants::{VOX_DEFAULT_FILE_NAME, VOX_FILE_EXT, VOX_FILE_FILTER_NAME};
use super::writer::{build_dot_vox_data, serialize};
use super::{ExportRequested, VoxFileSaver};

pub fn handle_export_request(
    mut commands: Commands,
    mut events: EventReader<ExportRequested>,
    grid: Option<Res<VoxelGrid>>,
    mask: Option<Res<VisibilityMask>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(_) = events.read().next() else {
        return;
    };
    events.clear();
    let (Some(grid), Some(mask)) = (grid, mask) else {
        warn!("export requested but VoxelGrid / VisibilityMask not present");
        return;
    };

    let data = match build_dot_vox_data(&grid, &mask) {
        Ok(d) => d,
        Err(e) => {
            error!("vox build failed: {:#}", e);
            return;
        }
    };
    let bytes = match serialize(&data) {
        Ok(b) => b,
        Err(e) => {
            error!("vox serialize failed: {:#}", e);
            return;
        }
    };
    info!(
        "vox payload: {} bytes, {} model(s), {} voxels",
        bytes.len(),
        data.models.len(),
        data.models.iter().map(|m| m.voxels.len()).sum::<usize>()
    );

    next_state.set(AppState::Exporting);
    commands
        .dialog()
        .add_filter(VOX_FILE_FILTER_NAME, &[VOX_FILE_EXT])
        .set_file_name(VOX_DEFAULT_FILE_NAME)
        .save_file::<VoxFileSaver>(bytes);
}

pub fn handle_save_complete(
    mut events: EventReader<DialogFileSaved<VoxFileSaver>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for ev in events.read() {
        match &ev.result {
            Ok(()) => info!("saved .vox to {}", ev.path.display()),
            Err(e) => error!("save failed for {}: {}", ev.path.display(), e),
        }
        next_state.set(AppState::Previewing);
    }
}

pub fn handle_save_canceled(
    mut events: EventReader<DialogFileSaveCanceled<VoxFileSaver>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if events.read().next().is_some() {
        info!("export canceled");
        events.clear();
        next_state.set(AppState::Previewing);
    }
}
