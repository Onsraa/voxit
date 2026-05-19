use bevy::color::{Color, LinearRgba};

use crate::source::constants::BIOME_PALETTE;

pub fn band_color(band: u8) -> Color {
    if band == 0 {
        return Color::BLACK;
    }
    let idx = ((band - 1) as usize).min(BIOME_PALETTE.len() - 1);
    let [r, g, b, _] = BIOME_PALETTE[idx];
    Color::srgb_u8(r, g, b)
}

pub fn band_linear(band: u8) -> [f32; 4] {
    let linear: LinearRgba = band_color(band).into();
    [linear.red, linear.green, linear.blue, linear.alpha]
}
