pub const ETC_OS_RELEASE: &str = "etc/os-release";
pub const USR_OS_RELEASE: &str = "usr/lib/os-release";
pub const APK_DB: &str = "lib/apk/db/installed";
pub const DPKG_STATUS: &str = "var/lib/dpkg/status";
pub const STATUS_D: &str = "var/lib/dpkg/status.d";
pub const STATUS_D_PREFIX: &str = "var/lib/dpkg/status.d/";

const LOCATIONS: [&str; 5] = [
    ETC_OS_RELEASE,
    USR_OS_RELEASE,
    APK_DB,
    DPKG_STATUS,
    STATUS_D,
];

const ANCESTORS: [&str; 6] = [
    "var",
    "var/lib",
    "var/lib/dpkg",
    STATUS_D,
    "lib/apk",
    "lib/apk/db",
];

pub fn is_ancestor(path: &str) -> bool {
    ANCESTORS.contains(&path)
}

pub fn covers(removed: &str, path: &str) -> bool {
    path == removed
        || path
            .strip_prefix(removed)
            .is_some_and(|rest| rest.starts_with('/'))
}

pub fn hides_below(dir: &str, path: &str) -> bool {
    dir.is_empty()
        || path
            .strip_prefix(dir)
            .is_some_and(|rest| rest.starts_with('/'))
}

pub fn removal_matters(removed: &str) -> bool {
    removed.starts_with(STATUS_D_PREFIX)
        || LOCATIONS.iter().any(|location| covers(removed, location))
}

pub fn opaque_matters(dir: &str) -> bool {
    dir == STATUS_D || LOCATIONS.iter().any(|location| hides_below(dir, location))
}
