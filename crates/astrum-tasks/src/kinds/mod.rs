pub mod compact;
pub mod expired;
pub mod purge_unused;
pub mod reconcile;

use std::sync::Arc;

use crate::Tasks;

pub const ALL: &str = "*";

pub fn register_builtin(tasks: &Tasks) {
    tasks.register(Arc::new(crate::cleanup::task::Cleanup));
    tasks.register(Arc::new(compact::Compact));
    tasks.register(Arc::new(reconcile::Reconcile));
    tasks.register(Arc::new(purge_unused::PurgeUnused));
    tasks.register(Arc::new(expired::PurgeExpired));
}

pub fn count(number: i64, one: &str, other: &str) -> String {
    format!("{number} {}", if number == 1 { one } else { other })
}

pub fn size(bytes: i64) -> String {
    const UNITS: [&str; 5] = ["bytes", "KB", "MB", "GB", "TB"];
    let mut value = bytes.max(0) as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    match unit {
        0 => format!("{bytes} bytes"),
        _ => format!("{value:.1} {}", UNITS[unit]),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sizes_read_well() {
        assert_eq!(super::size(12), "12 bytes");
        assert_eq!(super::size(2048), "2.0 KB");
        assert_eq!(super::size(5 * 1024 * 1024 + 1), "5.0 MB");
    }

    #[test]
    fn counts_agree_with_their_nouns() {
        assert_eq!(super::count(1, "blob", "blobs"), "1 blob");
        assert_eq!(super::count(0, "blob", "blobs"), "0 blobs");
        assert_eq!(super::count(2, "blob", "blobs"), "2 blobs");
    }
}
