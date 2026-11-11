pub mod cvss;
pub mod matcher;
pub mod osv;
pub mod queue;
pub mod version;
pub mod worker;

pub const PACKAGE_LOCK: i64 = i64::from_be_bytes(*b"astrpkgs");
