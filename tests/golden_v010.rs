// Locked-in baseline shape stats for the synthetic 128×128 fixture at default
// settings. Any optimization phase that changes these numbers is either a bug
// or a deliberate algorithmic change (greedy meshing changes triangle topology
// but visible voxel count stays the same).

use voxit::source::{RawVolume, ThresholdConfig};
use voxit::ui::resources::PreviewSettings;
use voxit::visibility::VisibilityMask;
use voxit::volume::{build_from_geotiff, MeshColorMode, VoxelGrid};

fn fixture() -> (VoxelGrid, PreviewSettings, VisibilityMask) {
    let side = 128_u32;
    let amp = 2500.0_f32;
    let sf = side as f32;
    let total = (side * side) as usize;
    let mut data = Vec::with_capacity(total);
    for y in 0..side {
        for x in 0..side {
            let fx = (x as f32) / sf;
            let fy = (y as f32) / sf;
            let pattern = (fx * 8.0).sin() * (fy * 6.0).cos();
            data.push(amp * 0.5 + amp * 0.5 * pattern);
        }
    }
    let raw = RawVolume {
        data,
        dims: [side, side, 1],
        spacing: [1.0 / 3600.0, 1.0 / 3600.0, 1.0],
        origin: [0.0, 0.0, 0.0],
    };
    let thresholds = ThresholdConfig {
        min: 0.0,
        max: amp,
    };
    let grid = build_from_geotiff(&raw, 30.0, 1.0, &thresholds);
    let settings = PreviewSettings {
        density_m_per_voxel: 30.0,
        threshold_min: 0.0,
        threshold_max: amp,
        elev_full_min: 0.0,
        elev_full_max: amp,
        crop_x: [0.0, 1.0],
        crop_y: [0.0, 1.0],
        crop_z: [0.0, 1.0],
        grid_dims: grid.dims,
        sea_level_m: 0.0,
        vertical_exaggeration: 1.0,
        mesh_voxels_per_axis: 64,
        mesh_yaw_quarters: 0,
        mesh_pitch_quarters: 0,
        mesh_color_mode: MeshColorMode::Auto,
        mesh_longest_axis_m: 1.0,
    };
    let mask = VisibilityMask::compute(&grid, &settings);
    (grid, settings, mask)
}

#[test]
fn grid_dims_match_v010_baseline() {
    let (grid, _, _) = fixture();
    assert_eq!(
        grid.dims,
        [132, 84, 132],
        "grid dims changed; expected the v0.1.0 baseline of [132, 84, 132]"
    );
}

#[test]
fn visible_voxel_count_matches_v010_baseline() {
    let (_, _, mask) = fixture();
    // Locked-in count: drift means a behavior change in classify, build_from_geotiff,
    // or VisibilityMask::compute. Re-bake intentionally if the change is expected.
    let count = mask.visible_count();
    let expected = 721_400_u32;
    let tolerance = 200_u32;
    assert!(
        count.abs_diff(expected) < tolerance,
        "visible voxels {} differs from baseline {} by more than tolerance {}",
        count,
        expected,
        tolerance
    );
}
