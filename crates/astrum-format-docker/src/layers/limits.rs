const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_output: u64,
    pub ratio_floor: u64,
    pub max_ratio: u64,
    pub max_frames: u64,
    pub max_window: u64,
    pub max_header: usize,
    pub max_entries: u64,
    pub max_file: u64,
    pub max_name: usize,
    pub max_meta: u64,
    pub max_removed: usize,
    pub max_status_files: usize,
    pub max_packages: usize,
}

impl Limits {
    pub const DEFAULT: Limits = Limits {
        max_output: 4 * GIB,
        ratio_floor: 64 * MIB,
        max_ratio: 2048,
        max_frames: 1_000_000,
        max_window: 128 * MIB,
        max_header: 64 * 1024,
        max_entries: 1_000_000,
        max_file: 64 * MIB,
        max_name: 4096,
        max_meta: 64 * 1024,
        max_removed: 4096,
        max_status_files: 4096,
        max_packages: astrum_sbom::MAX_PACKAGES,
    };
}
