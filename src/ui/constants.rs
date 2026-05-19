use bevy::color::Color;

pub const STATE_TEXT_IDLE: &str = "Drop a file";
pub const STATE_TEXT_LOADING: &str = "Loading…";
pub const STATE_TEXT_PREVIEWING: &str = "Previewing";
pub const STATE_TEXT_EXPORTING: &str = "Exporting…";

pub const STATE_TEXT_FONT_SIZE: f32 = 48.0;
pub const STATE_TEXT_COLOR: Color = Color::srgb(0.92, 0.92, 0.92);
pub const BACKGROUND_COLOR: Color = Color::srgb(0.08, 0.08, 0.10);
