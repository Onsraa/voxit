use std::time::Duration;

use bevy::pbr::{AmbientLight, DirectionalLight, DirectionalLightBundle};
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::source::geotiff::GeoTiffSource;
use crate::source::{RawVolume, VolumeSource};
use crate::ui::resources::{
    BiomeMode, MeshDirty, PreviewSettings, PreviewStats, VolumeDebounce, VolumeDirty,
};
use crate::visibility::VisibilityMask;
use crate::volume::{build_from_geotiff, VoxelGrid};

use crate::volume::estimate_pixel_spacing_m;

use super::components::{PreviewCamera, PreviewLight, PreviewMesh};
use super::constants::{
    AMBIENT_BRIGHTNESS, AMBIENT_COLOR, CLEAR_COLOR, DEFAULT_DENSITY_M_PER_VOXEL,
    LIGHT_EULER_PITCH, LIGHT_EULER_YAW, LIGHT_ILLUMINANCE,
};
use super::surface_mesh::build_surface_mesh;

pub const VOLUME_DEBOUNCE_MS: u64 = 300;

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
    mut stats: ResMut<PreviewStats>,
) {
    let thresholds = GeoTiffSource::default_thresholds(&raw);
    let density = DEFAULT_DENSITY_M_PER_VOXEL;
    let vert_exag = 1.0_f32;
    let grid = build_from_geotiff(&raw, density, vert_exag, &thresholds);

    let settings = PreviewSettings {
        density_m_per_voxel: density,
        threshold_min: thresholds.min,
        threshold_max: thresholds.max,
        elev_full_min: thresholds.min,
        elev_full_max: thresholds.max,
        crop_x: [0.0, 1.0],
        crop_y: [0.0, 1.0],
        crop_z: [0.0, 1.0],
        grid_dims: grid.dims,
        sea_level_m: thresholds.min,
        vertical_exaggeration: vert_exag,
        biome_mode: BiomeMode::Elevation,
    };

    let mask = VisibilityMask::compute(&grid, &settings);
    let mesh = build_surface_mesh(&grid, &mask);

    let triangle_count = (mesh.indices().map(|i| i.len()).unwrap_or(0) / 3) as u32;
    *stats = PreviewStats {
        visible_voxels: mask.visible_count(),
        triangle_count,
    };

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

    let pixel_spacing_m = estimate_pixel_spacing_m(&raw);
    let world_x = raw.dims[0] as f32 * pixel_spacing_m;
    let world_z = raw.dims[1] as f32 * pixel_spacing_m;
    let world_y = (thresholds.max - thresholds.min) * vert_exag;

    commands.spawn((
        PbrBundle {
            mesh: mesh_handle,
            material: material_handle,
            transform: Transform::from_scale(Vec3::splat(density)),
            ..default()
        },
        PreviewMesh,
    ));

    let focus = Vec3::new(world_x * 0.5, world_y * 0.25, world_z * 0.5);
    let span = world_x.max(world_z);
    let cam_pos = focus + Vec3::new(span * 0.7, span * 0.6, span * 0.7);
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(cam_pos).looking_at(focus, Vec3::Y),
            ..default()
        },
        PanOrbitCamera {
            focus,
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
    commands.insert_resource(settings);
}

pub fn teardown_preview(
    mut commands: Commands,
    meshes: Query<Entity, With<PreviewMesh>>,
    cameras: Query<Entity, With<PreviewCamera>>,
    lights: Query<Entity, With<PreviewLight>>,
    mut debounce: ResMut<VolumeDebounce>,
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
    commands.remove_resource::<PreviewSettings>();
    debounce.timer = None;
}

pub fn rebuild_mesh_on_dirty(
    mut events: EventReader<MeshDirty>,
    grid: Option<Res<VoxelGrid>>,
    settings: Option<Res<PreviewSettings>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_query: Query<&Handle<Mesh>, With<PreviewMesh>>,
    mut stats: ResMut<PreviewStats>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();
    let Some(grid) = grid else { return };
    let Some(settings) = settings else { return };
    let Ok(handle) = mesh_query.get_single() else {
        return;
    };

    let mask = VisibilityMask::compute(&grid, &settings);
    let mesh = build_surface_mesh(&grid, &mask);
    let triangle_count = (mesh.indices().map(|i| i.len()).unwrap_or(0) / 3) as u32;
    *stats = PreviewStats {
        visible_voxels: mask.visible_count(),
        triangle_count,
    };

    if let Some(slot) = meshes.get_mut(handle) {
        *slot = mesh;
    }
    commands.insert_resource(mask);
}

pub fn volume_dirty_starts_debounce(
    mut events: EventReader<VolumeDirty>,
    mut debounce: ResMut<VolumeDebounce>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();
    debounce.timer = Some(Timer::new(
        Duration::from_millis(VOLUME_DEBOUNCE_MS),
        TimerMode::Once,
    ));
}

pub fn rebuild_volume_after_debounce(
    time: Res<Time>,
    mut debounce: ResMut<VolumeDebounce>,
    raw: Option<Res<RawVolume>>,
    settings: Option<Res<PreviewSettings>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_query: Query<(&Handle<Mesh>, &mut Transform), With<PreviewMesh>>,
    mut stats: ResMut<PreviewStats>,
) {
    let Some(timer) = debounce.timer.as_mut() else {
        return;
    };
    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }
    debounce.timer = None;

    let Some(raw) = raw else { return };
    let Some(mut settings_owned) = settings.map(|s| s.clone()) else {
        return;
    };
    let Ok((handle, mut transform)) = mesh_query.get_single_mut() else {
        return;
    };

    let thresholds = crate::source::ThresholdConfig {
        min: settings_owned.elev_full_min,
        max: settings_owned.elev_full_max,
    };
    let grid = build_from_geotiff(
        &raw,
        settings_owned.density_m_per_voxel,
        settings_owned.vertical_exaggeration,
        &thresholds,
    );

    settings_owned.grid_dims = grid.dims;

    let mask = VisibilityMask::compute(&grid, &settings_owned);
    let mesh = build_surface_mesh(&grid, &mask);
    let triangle_count = (mesh.indices().map(|i| i.len()).unwrap_or(0) / 3) as u32;
    *stats = PreviewStats {
        visible_voxels: mask.visible_count(),
        triangle_count,
    };

    if let Some(slot) = meshes.get_mut(handle) {
        *slot = mesh;
    }
    transform.scale = Vec3::splat(settings_owned.density_m_per_voxel);
    commands.insert_resource(grid);
    commands.insert_resource(mask);
    commands.insert_resource(settings_owned);
}
