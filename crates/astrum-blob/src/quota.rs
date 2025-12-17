use serde::{Deserialize, Serialize};

use crate::Totals;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuotaKind {
    SpaceRemainingQuota,
    SpaceUsedQuota,
}

impl QuotaKind {
    pub fn as_str(self) -> &'static str {
        match self {
            QuotaKind::SpaceRemainingQuota => "spaceRemainingQuota",
            QuotaKind::SpaceUsedQuota => "spaceUsedQuota",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "spaceRemainingQuota" => Some(QuotaKind::SpaceRemainingQuota),
            "spaceUsedQuota" => Some(QuotaKind::SpaceUsedQuota),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftQuota {
    #[serde(rename = "type")]
    pub kind: QuotaKind,
    pub limit: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuotaStatus {
    pub violation: bool,
    pub message: String,
}

impl SoftQuota {
    pub fn check(&self, store: &str, totals: Totals, available: Option<u64>) -> QuotaStatus {
        let limit = human_size(self.limit);
        match self.kind {
            QuotaKind::SpaceUsedQuota => QuotaStatus {
                violation: totals.total_bytes > self.limit,
                message: format!(
                    "Blob store {store} uses {} of its {limit} limit",
                    human_size(totals.total_bytes)
                ),
            },
            QuotaKind::SpaceRemainingQuota => match available {
                Some(available) => QuotaStatus {
                    violation: available < self.limit,
                    message: format!(
                        "Blob store {store} should keep {limit} free and has {} free",
                        human_size(available)
                    ),
                },
                None => QuotaStatus {
                    violation: false,
                    message: format!(
                        "Blob store {store} should keep {limit} free and its space has no limit"
                    ),
                },
            },
        }
    }
}

pub fn status(
    store: &str,
    quota: Option<&SoftQuota>,
    totals: Totals,
    available: Option<u64>,
) -> QuotaStatus {
    match quota {
        Some(quota) => quota.check(store, totals, available),
        None => QuotaStatus {
            violation: false,
            message: format!("Blob store {store} has no quota"),
        },
    }
}

pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["KB", "MB", "GB", "TB", "PB", "EB"];
    if bytes < 1000 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64 / 1000.0;
    let mut unit = 0;
    while value >= 999.995 && unit + 1 < UNITS.len() {
        value /= 1000.0;
        unit += 1;
    }
    format!("{value:.2} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn used(bytes: u64) -> Totals {
        Totals {
            blob_count: 1,
            total_bytes: bytes,
        }
    }

    #[test]
    fn sizes_read_in_decimal_units() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(999), "999 B");
        assert_eq!(human_size(1_048_576), "1.05 MB");
        assert_eq!(human_size(13_370_000_000), "13.37 GB");
        assert_eq!(human_size(999_999_999_999_999), "1.00 PB");
        assert_eq!(human_size(u64::MAX), "18.45 EB");
    }

    #[test]
    fn a_used_space_quota_is_violated_above_its_limit() {
        let quota = SoftQuota {
            kind: QuotaKind::SpaceUsedQuota,
            limit: 1000,
        };
        assert!(!quota.check("a", used(1000), None).violation);
        let over = quota.check("a", used(1001), Some(5));
        assert!(over.violation);
        assert_eq!(
            over.message,
            "Blob store a uses 1.00 KB of its 1.00 KB limit"
        );
    }

    #[test]
    fn a_remaining_space_quota_is_violated_below_its_limit() {
        let quota = SoftQuota {
            kind: QuotaKind::SpaceRemainingQuota,
            limit: 2_000_000,
        };
        assert!(!quota.check("b", used(0), Some(2_000_000)).violation);
        let short = quota.check("b", used(0), Some(1_500_000));
        assert!(short.violation);
        assert_eq!(
            short.message,
            "Blob store b should keep 2.00 MB free and has 1.50 MB free"
        );
        assert!(
            !quota.check("b", used(0), None).violation,
            "unlimited space"
        );
    }

    #[test]
    fn a_store_without_a_quota_never_violates() {
        let status = status("c", None, used(u64::MAX), Some(0));
        assert!(!status.violation);
        assert_eq!(status.message, "Blob store c has no quota");
    }

    #[test]
    fn quotas_keep_their_documented_json() {
        let quota = SoftQuota {
            kind: QuotaKind::SpaceRemainingQuota,
            limit: 5,
        };
        let json = serde_json::to_value(quota).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"type": "spaceRemainingQuota", "limit": 5})
        );
        assert_eq!(serde_json::from_value::<SoftQuota>(json).unwrap(), quota);
        assert_eq!(
            QuotaKind::parse("spaceUsedQuota"),
            Some(QuotaKind::SpaceUsedQuota)
        );
        assert_eq!(QuotaKind::parse("SPACEUSEDQUOTA"), None);
    }
}
