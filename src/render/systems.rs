use bevy::pbr::{AmbientLight, DirectionalLight, DirectionalLightBundle};
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::source::geotiff::GeoTiffSource;
use crate::source::{RawVolume, VolumeSource};
use crate::visibility::VisibilityMask;
use crate::volume::{build_from_geotiff, VoxelGrid};

use super::components::{PreviewCamera, PreviewLight, PreviewMesh};
use super::constants::{
    AMBIENT_BRIGHTNESS, AMBIENT_COLOR, CAMERA_LOOK_AT, CAMERA_START_POS, CLEAR_COLOR,
    DEFAULT_DENSITY_M_PER_VOXEL, LIGHT_EULER_PITCH, LIGHT_EULER_YAW, LIGHT_ILLUMINANCE,
};
use super::surface_mesh::build_surface_mesh;

pub fn setup_ambient(mut commands: Commands) {
    commands.insert_resource(ClearColor(CLEAR_COLOR));
    commands.insert_resource(AmbientLight {
        color: AMBIENT_COLOR,
        brightness: AMBIENT_BRIGHTNESS,
    });
}

pub fn build_preview_on_enter(
    mut commands: Commands,
    raw: Res<RawVolume>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let thresholds = GeoTiffSource::default_thresholds(&raw);
    let grid = build_from_geotiff(&raw, DEFAULT_DENSITY_M_PER_VOXEL, &thresholds);
    let mask = VisibilityMask::from_grid(&grid);
    let mesh = build_surface_mesh(&grid, &mask);

    info!(
        "VoxelGrid {:?} ({} cells) → mesh with {} vertices",
        grid.dims,
        grid.data.len(),
        mesh.count_vertices()
    );

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: mesh_handle,
            material: material_handle,
            ..default()
        },
        PreviewMesh,
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(CAMERA_START_POS)
                .looking_at(CAMERA_LOOK_AT, Vec3::Y),
            ..default()
        },
        PanOrbitCamera {
            focus: CAMERA_LOOK_AT,
            ..default()
        },
        PreviewCamera,
    ));

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: LIGHT_ILLUMINANCE,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_euler(
                EulerRot::YXZ,
                LIGHT_EULER_YAW,
                LIGHT_EULER_PITCH,
                0.0,
            )),
            ..default()
        },
        PreviewLight,
    ));

    commands.insert_resource(grid);
    commands.insert_resource(mask);
}

pub fn teardown_preview(
    mut commands: Commands,
    meshes: Query<Entity, With<PreviewMesh>>,
    cameras: Query<Entity, With<PreviewCamera>>,
    lights: Query<Entity, With<PreviewLight>>,
) {
    for e in &meshes {
        commands.entity(e).despawn();
    }
    for e in &cameras {
        commands.entity(e).despawn();
    }
    for e in &lights {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<VoxelGrid>();
    commands.remove_resource::<VisibilityMask>();
}
