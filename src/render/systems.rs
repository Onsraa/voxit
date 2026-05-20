use std::time::Duration;

use bevy::pbr::{AmbientLight, DirectionalLight, DirectionalLightBundle};
use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::source::geotiff::GeoTiffSource;
use crate::source::mesh::MeshData;
use crate::source::{RawVolume, SourceData, VolumeSource};
use crate::tasks::spawn_async;
use crate::ui::resources::{
    MeshDirty, MeshRebuildPending, PreviewSettings, PreviewStats, VolumeDebounce, VolumeDirty,
};
use crate::visibility::VisibilityMask;
use crate::volume::estimate_pixel_spacing_m;
use crate::volume::{build_from_geotiff, build_from_mesh, MeshColorMode, VoxelGrid};

use super::components::{
    MeshRebuildResult, MeshRebuildTask, PreviewCamera, PreviewLight, PreviewMesh,
};
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
    source: Res<SourceData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut stats: ResMut<PreviewStats>,
) {
    let (grid, settings, world_extent) = match &*source {
        SourceData::Heightmap(raw) => build_heightmap_preview(raw),
        SourceData::Mesh(mesh) => build_mesh_preview(mesh),
    };

    let mask = VisibilityMask::compute(&grid, &settings);
    let mesh_data = build_surface_mesh(&grid, &mask);

    let triangle_count = (mesh_data.indices().map(|i| i.len()).unwrap_or(0) / 3) as u32;
    *stats = PreviewStats {
        visible_voxels: mask.visible_count(),
        triangle_count,
    };

    info!(
        "VoxelGrid {:?} ({} cells) → mesh with {} vertices",
        grid.dims,
        grid.data.len(),
        mesh_data.count_vertices()
    );

    let mesh_handle = meshes.add(mesh_data);
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
            transform: Transform::from_scale(Vec3::splat(settings.density_m_per_voxel)),
            ..default()
        },
        PreviewMesh,
    ));

    let focus = Vec3::new(
        world_extent[0] * 0.5,
        world_extent[1] * 0.25,
        world_extent[2] * 0.5,
    );
    let span = world_extent[0].max(world_extent[2]);
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

fn build_heightmap_preview(raw: &RawVolume) -> (VoxelGrid, PreviewSettings, [f32; 3]) {
    let thresholds = GeoTiffSource::default_thresholds(raw);
    let density = DEFAULT_DENSITY_M_PER_VOXEL;
    let vert_exag = 1.0_f32;
    let grid = build_from_geotiff(raw, density, vert_exag, &thresholds);

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
        mesh_voxels_per_axis: 64,
        mesh_yaw_quarters: 0,
        mesh_pitch_quarters: 0,
        mesh_color_mode: MeshColorMode::Auto,
        mesh_longest_axis_m: 1.0,
    };

    let pixel_spacing_m = estimate_pixel_spacing_m(raw);
    let world_extent = [
        raw.dims[0] as f32 * pixel_spacing_m,
        (thresholds.max - thresholds.min) * vert_exag,
        raw.dims[1] as f32 * pixel_spacing_m,
    ];

    (grid, settings, world_extent)
}

fn build_mesh_preview(mesh: &MeshData) -> (VoxelGrid, PreviewSettings, [f32; 3]) {
    let extent = [
        mesh.aabb_max[0] - mesh.aabb_min[0],
        mesh.aabb_max[1] - mesh.aabb_min[1],
        mesh.aabb_max[2] - mesh.aabb_min[2],
    ];
    let longest = extent[0].max(extent[1]).max(extent[2]).max(1e-3);
    let voxels_per_axis = 64_u32;
    let voxel_size = (longest / voxels_per_axis as f32).max(1e-6);

    let grid = build_from_mesh(mesh, voxel_size, MeshColorMode::Auto, 0, 0);

    let elev_min = mesh.aabb_min[1];
    let elev_max = mesh.aabb_max[1];

    let settings = PreviewSettings {
        density_m_per_voxel: voxel_size,
        threshold_min: elev_min,
        threshold_max: elev_max,
        elev_full_min: elev_min,
        elev_full_max: elev_max,
        crop_x: [0.0, 1.0],
        crop_y: [0.0, 1.0],
        crop_z: [0.0, 1.0],
        grid_dims: grid.dims,
        sea_level_m: elev_min,
        vertical_exaggeration: 1.0,
        mesh_voxels_per_axis: voxels_per_axis,
        mesh_yaw_quarters: 0,
        mesh_pitch_quarters: 0,
        mesh_color_mode: MeshColorMode::Auto,
        mesh_longest_axis_m: longest,
    };

    (grid, settings, extent)
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

pub fn schedule_mesh_rebuild(
    mut commands: Commands,
    mut events: EventReader<MeshDirty>,
    grid: Option<Res<VoxelGrid>>,
    settings: Option<Res<PreviewSettings>>,
    in_flight: Query<&MeshRebuildTask>,
    mut pending: ResMut<MeshRebuildPending>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();

    if !in_flight.is_empty() {
        // A rebuild is already running. Mark that the latest settings are not
        // yet reflected — the poll system will re-fire MeshDirty on
        // completion so the final slider position always gets applied.
        pending.0 = true;
        return;
    }

    let Some(grid) = grid else { return };
    let Some(settings) = settings else { return };
    let grid_clone = grid.clone();
    let settings_clone = settings.clone();
    pending.0 = false;

    let task = spawn_async(move || {
        let mut mask = VisibilityMask::new_empty();
        mask.recompute(&grid_clone, &settings_clone);
        let mesh = build_surface_mesh(&grid_clone, &mask);
        let triangle_count = (mesh.indices().map(|i| i.len()).unwrap_or(0) / 3) as u32;
        MeshRebuildResult {
            mesh,
            mask,
            triangle_count,
        }
    });
    commands.spawn(MeshRebuildTask(task));
}

pub fn poll_mesh_rebuild(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut MeshRebuildTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_query: Query<&Handle<Mesh>, With<PreviewMesh>>,
    mut stats: ResMut<PreviewStats>,
    mut pending: ResMut<MeshRebuildPending>,
    mut mesh_events: EventWriter<MeshDirty>,
) {
    for (entity, mut task) in &mut tasks {
        let Some(result) = future::block_on(future::poll_once(&mut task.0)) else {
            continue;
        };
        let Ok(handle) = mesh_query.get_single() else {
            commands.entity(entity).despawn();
            continue;
        };

        *stats = PreviewStats {
            visible_voxels: result.mask.visible_count(),
            triangle_count: result.triangle_count,
        };

        if let Some(slot) = meshes.get_mut(handle) {
            *slot = result.mesh;
        }
        commands.insert_resource(result.mask);
        commands.entity(entity).despawn();

        if pending.0 {
            pending.0 = false;
            mesh_events.send(MeshDirty);
        }
    }
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
    source: Option<Res<SourceData>>,
    settings: Option<Res<PreviewSettings>>,
    mask: Option<ResMut<VisibilityMask>>,
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

    let Some(source) = source else { return };
    let Some(mut settings_owned) = settings.map(|s| s.clone()) else {
        return;
    };
    let Some(mut mask) = mask else { return };
    let Ok((handle, mut transform)) = mesh_query.get_single_mut() else {
        return;
    };

    let grid = match &*source {
        SourceData::Heightmap(raw) => {
            let thresholds = crate::source::ThresholdConfig {
                min: settings_owned.elev_full_min,
                max: settings_owned.elev_full_max,
            };
            build_from_geotiff(
                raw,
                settings_owned.density_m_per_voxel,
                settings_owned.vertical_exaggeration,
                &thresholds,
            )
        }
        SourceData::Mesh(mesh) => build_from_mesh(
            mesh,
            settings_owned.density_m_per_voxel,
            settings_owned.mesh_color_mode,
            settings_owned.mesh_yaw_quarters,
            settings_owned.mesh_pitch_quarters,
        ),
    };

    settings_owned.grid_dims = grid.dims;

    mask.recompute(&grid, &settings_owned);
    let new_mesh = build_surface_mesh(&grid, &mask);
    let triangle_count = (new_mesh.indices().map(|i| i.len()).unwrap_or(0) / 3) as u32;
    *stats = PreviewStats {
        visible_voxels: mask.visible_count(),
        triangle_count,
    };

    if let Some(slot) = meshes.get_mut(handle) {
        *slot = new_mesh;
    }
    transform.scale = Vec3::splat(settings_owned.density_m_per_voxel);
    commands.insert_resource(grid);
    commands.insert_resource(settings_owned);
}
