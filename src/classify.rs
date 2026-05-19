// Elevation → palette band index. Band 0 is reserved for "empty"; biome bands
// occupy 1..=BAND_COUNT.

pub const BAND_COUNT: u8 = 8;

pub fn classify_band(elev: f32, min: f32, max: f32) -> u8 {
    if !elev.is_finite() || max <= min {
        return 0;
    }
    let norm = ((elev - min) / (max - min)).clamp(0.0, 1.0);
    let band = (norm * BAND_COUNT as f32).floor() as u8;
    band.min(BAND_COUNT - 1) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nan_returns_empty() {
        assert_eq!(classify_band(f32::NAN, 0.0, 100.0), 0);
    }

    #[test]
    fn degenerate_range_returns_empty() {
        assert_eq!(classify_band(50.0, 100.0, 100.0), 0);
    }

    #[test]
    fn min_maps_to_band_one() {
        assert_eq!(classify_band(0.0, 0.0, 100.0), 1);
    }

    #[test]
    fn max_maps_to_top_band() {
        assert_eq!(classify_band(100.0, 0.0, 100.0), BAND_COUNT);
    }

    #[test]
    fn midpoint_maps_to_middle_band() {
        assert_eq!(classify_band(50.0, 0.0, 100.0), 5);
    }
}
