// Configuration constants matching the C reference implementation

pub const MAX_NUM_SOURCES: usize = 4;
pub const MUXER_OUTPUT_WIDTH: u32 = 1920;
pub const MUXER_OUTPUT_HEIGHT: u32 = 1080;
pub const TILED_OUTPUT_WIDTH: u32 = 1280;
pub const TILED_OUTPUT_HEIGHT: u32 = 720;
pub const GPU_ID: u32 = 0;

pub const TILER_ROWS: u32 = 2;
pub const TILER_COLUMNS: u32 = 2;

pub const PGIE_CONFIG_FILE: &str = "dstest_pgie_config.txt";
pub const TRACKER_CONFIG_FILE: &str = "dstest_tracker_config.txt";
pub const SGIE1_CONFIG_FILE: &str = "dstest_sgie1_config.txt";
pub const SGIE2_CONFIG_FILE: &str = "dstest_sgie2_config.txt";
pub const SGIE3_CONFIG_FILE: &str = "dstest_sgie3_config.txt";

pub const SOURCE_ADD_INTERVAL_SECS: u64 = 10;
pub const SOURCE_DELETE_INTERVAL_SECS: u64 = 10;