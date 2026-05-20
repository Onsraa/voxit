use bevy::prelude::*;
use bevy::tasks::Task;

use crate::visibility::VisibilityMask;

#[derive(Component)]
pub struct PreviewMesh;

#[derive(Component)]
pub struct PreviewCamera;

#[derive(Component)]
pub struct PreviewLight;

#[derive(Component)]
pub struct MeshRebuildTask(pub Task<MeshRebuildResult>);

pub struct MeshRebuildResult {
    pub mesh: Mesh,
    pub mask: VisibilityMask,
    pub triangle_count: u32,
}
