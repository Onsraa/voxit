use anyhow::Result;
use bevy::prelude::*;
use bevy::tasks::Task;

use super::SourceData;

#[derive(Component)]
pub struct ParseTask(pub Task<Result<SourceData>>);
