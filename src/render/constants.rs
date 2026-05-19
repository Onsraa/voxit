use bevy::color::Color;
use bevy::math::Vec3;

pub const DEFAULT_DENSITY_M_PER_VOXEL: f32 = 20.0;

pub const CAMERA_START_POS: Vec3 = Vec3::new(220.0, 220.0, 220.0);
pub const CAMERA_LOOK_AT: Vec3 = Vec3::new(64.0, 40.0, 64.0);

pub const LIGHT_ILLUMINANCE: f32 = 8000.0;
pub const LIGHT_EULER_PITCH: f32 = -0.9;
pub const LIGHT_EULER_YAW: f32 = -0.6;

pub const AMBIENT_COLOR: Color = Color::srgb(0.30, 0.32, 0.36);
pub const AMBIENT_BRIGHTNESS: f32 = 350.0;

pub const CLEAR_COLOR: Color = Color::srgb(0.04, 0.05, 0.07);
