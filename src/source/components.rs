use anyhow::Result;
use bevy::prelude::*;
use bevy::tasks::Task;

use super::RawVolume;

#[derive(Component)]
pub struct ParseTask(pub Task<Result<RawVolume>>);
