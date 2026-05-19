// SRTM-derived int16 GeoTIFFs use -32768 as the NODATA sentinel. Cast to f32
// then compare exactly — the value is integral so float compare is safe.
pub const GEOTIFF_NODATA_SRTM: f32 = -32768.0;

// Default 8-band elevation palette. Index 0 reserved for "empty" in classify.
pub const BIOME_PALETTE: [[u8; 4]; 8] = [
    [30, 80, 160, 255],   // deep water
    [80, 130, 200, 255],  // shallow water
    [220, 200, 140, 255], // beach
    [60, 140, 60, 255],   // lowland grass
    [40, 100, 40, 255],   // forest
    [120, 100, 60, 255],  // hill
    [180, 170, 160, 255], // mountain
    [240, 240, 250, 255], // snow
];
