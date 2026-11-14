const KIB: usize = 1024;
const MIB: usize = 1024 * KIB;
const GIB: u64 = 1024 * MIB as u64;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_output: u64,
    pub stall_input: u64,
    pub stall_output: u64,
    pub max_frames: u64,
    pub max_window: u64,
    pub max_header: usize,
    pub max_entries: u64,
    pub max_file: u64,
    pub max_name: usize,
    pub max_meta: u64,
    pub max_removed: usize,
    pub max_status_files: usize,
    pub max_db_packages: usize,
    pub facts_budget: usize,
    pub merged_budget: usize,
    pub cache_limit: usize,
    pub max_packages: usize,
}

impl Limits {
    pub const DEFAULT: Limits = Limits {
        max_output: 4 * GIB,
        stall_input: 256 * MIB as u64,
        stall_output: MIB as u64,
        max_frames: 1_000_000,
        max_window: 128 * MIB as u64,
        max_header: 64 * KIB,
        max_entries: 1_000_000,
        max_file: 64 * MIB as u64,
        max_name: 4096,
        max_meta: 64 * KIB as u64,
        max_removed: 4096,
        max_status_files: 4096,
        max_db_packages: 20_000,
        facts_budget: 8 * MIB,
        merged_budget: 16 * MIB,
        cache_limit: 8 * MIB,
        max_packages: astrum_sbom::MAX_PACKAGES,
    };
}
