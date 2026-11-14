pub mod feed;
pub mod import;
pub mod purge;
pub mod record;
pub mod source;

pub use feed::{EcosystemLock, Feed, Outcome, Stale, SyncError, feeds, stale, sync, sync_with};
pub use import::{ImportError, Stats};
pub use source::{AdvisorySource, Fetched, HttpDump, SourceError, TempFile, Upload};

pub const DEFAULT_BASE_URL: &str = "https://osv-vulnerabilities.storage.googleapis.com";
pub const DEFAULT_ECOSYSTEMS: &str = "Maven,npm,PyPI,Go,crates.io,NuGet,Alpine,Debian";
pub const DEFAULT_STALE_AFTER_DAYS: i64 = 7;
pub const MAX_ARCHIVE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const MAX_ECOSYSTEM_CHARS: usize = 64;

pub fn valid_ecosystem(name: &str) -> bool {
    let mut chars = name.chars();
    name.len() <= MAX_ECOSYSTEM_CHARS
        && chars
            .next()
            .is_some_and(|first| first.is_ascii_alphanumeric())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | ':' | '+' | '-' | '_'))
}

pub fn normalise_name(ecosystem: &str, name: &str) -> String {
    match ecosystem {
        "PyPI" => {
            let mut out = String::with_capacity(name.len());
            let mut separated = false;
            for c in name.chars() {
                if matches!(c, '-' | '_' | '.') {
                    separated = true;
                    continue;
                }
                if separated && !out.is_empty() {
                    out.push('-');
                }
                separated = false;
                out.extend(c.to_lowercase());
            }
            out
        }
        "NuGet" => name.to_lowercase(),
        _ => name.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecosystem_names_are_checked() {
        for good in ["npm", "crates.io", "Alpine:v3.19", "Ubuntu:22.04:LTS"] {
            assert!(valid_ecosystem(good), "{good}");
        }
        for bad in [
            "",
            "../npm",
            "a/b",
            "a b",
            ".hidden",
            "x%2f",
            &"a".repeat(65),
        ] {
            assert!(!valid_ecosystem(bad), "{bad}");
        }
    }

    #[test]
    fn names_are_normalised_per_ecosystem() {
        assert_eq!(
            normalise_name("PyPI", "Django_Rest.Framework"),
            "django-rest-framework"
        );
        assert_eq!(normalise_name("PyPI", "a--b__c"), "a-b-c");
        assert_eq!(
            normalise_name("NuGet", "Newtonsoft.JSON"),
            "newtonsoft.json"
        );
        assert_eq!(normalise_name("npm", "@Scope/Name"), "@Scope/Name");
        assert_eq!(normalise_name("Go", "github.com/A/B"), "github.com/A/B");
    }
}
