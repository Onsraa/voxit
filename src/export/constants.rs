// .vox per-model hard limit. Single-model export rejects any axis exceeding
// this; multi-model chunking is Phase 7.
pub const VOX_MAX_DIM: u32 = 256;

pub const VOX_FILE_FILTER_NAME: &str = "MagicaVoxel";
pub const VOX_FILE_EXT: &str = "vox";
pub const VOX_DEFAULT_FILE_NAME: &str = "terrain.vox";

pub const DOT_VOX_VERSION: u32 = 150;
