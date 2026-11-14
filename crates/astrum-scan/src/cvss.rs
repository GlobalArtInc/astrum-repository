use crate::version::MAX_LEN;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Unknown,
}

impl Severity {
    pub fn from_score(score: f64) -> Severity {
        if score >= 9.0 {
            Severity::Critical
        } else if score >= 7.0 {
            Severity::High
        } else if score >= 4.0 {
            Severity::Medium
        } else if score >= 0.1 {
            Severity::Low
        } else {
            Severity::Unknown
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
            Severity::Unknown => "unknown",
        }
    }
}

const METRICS: [&str; 8] = ["AV", "AC", "PR", "UI", "S", "C", "I", "A"];

const OPTIONAL: [(&str, &[&str]); 14] = [
    ("E", &["X", "H", "F", "P", "U"]),
    ("RL", &["X", "O", "T", "W", "U"]),
    ("RC", &["X", "C", "R", "U"]),
    ("CR", &["X", "H", "M", "L"]),
    ("IR", &["X", "H", "M", "L"]),
    ("AR", &["X", "H", "M", "L"]),
    ("MAV", &["X", "N", "A", "L", "P"]),
    ("MAC", &["X", "L", "H"]),
    ("MPR", &["X", "N", "L", "H"]),
    ("MUI", &["X", "N", "R"]),
    ("MS", &["X", "U", "C"]),
    ("MC", &["X", "H", "L", "N"]),
    ("MI", &["X", "H", "L", "N"]),
    ("MA", &["X", "H", "L", "N"]),
];

pub fn base_score(vector: &str) -> Option<f64> {
    if vector.len() > MAX_LEN {
        return None;
    }
    let rest = vector
        .strip_prefix("CVSS:3.1/")
        .or_else(|| vector.strip_prefix("CVSS:3.0/"))?;
    let mut values = [None; 8];
    let mut seen: Vec<&str> = Vec::new();
    for part in rest.split('/') {
        let (key, value) = part.split_once(':')?;
        if seen.contains(&key) {
            return None;
        }
        seen.push(key);
        match METRICS.iter().position(|metric| *metric == key) {
            Some(index) => values[index] = Some(value),
            None => {
                let (_, allowed) = OPTIONAL.iter().find(|(name, _)| *name == key)?;
                if !allowed.contains(&value) {
                    return None;
                }
            }
        }
    }
    let mut picked = [""; 8];
    for (slot, value) in picked.iter_mut().zip(values) {
        *slot = value?;
    }
    let [
        av,
        ac,
        pr,
        ui,
        scope,
        confidentiality,
        integrity,
        availability,
    ] = picked;

    let attack_vector = match av {
        "N" => 0.85,
        "A" => 0.62,
        "L" => 0.55,
        "P" => 0.2,
        _ => return None,
    };
    let attack_complexity = match ac {
        "L" => 0.77,
        "H" => 0.44,
        _ => return None,
    };
    let changed = match scope {
        "U" => false,
        "C" => true,
        _ => return None,
    };
    let privileges = match (pr, changed) {
        ("N", _) => 0.85,
        ("L", false) => 0.62,
        ("L", true) => 0.68,
        ("H", false) => 0.27,
        ("H", true) => 0.5,
        _ => return None,
    };
    let user_interaction = match ui {
        "N" => 0.85,
        "R" => 0.62,
        _ => return None,
    };
    let impact_weight = |value: &str| -> Option<f64> {
        match value {
            "H" => Some(0.56),
            "L" => Some(0.22),
            "N" => Some(0.0),
            _ => None,
        }
    };
    let c = impact_weight(confidentiality)?;
    let i = impact_weight(integrity)?;
    let a = impact_weight(availability)?;

    let isc = 1.0 - (1.0 - c) * (1.0 - i) * (1.0 - a);
    let impact = if changed {
        7.52 * (isc - 0.029) - 3.25 * (isc - 0.02).powi(15)
    } else {
        6.42 * isc
    };
    let exploitability = 8.22 * attack_vector * attack_complexity * privileges * user_interaction;
    if impact <= 0.0 {
        return Some(0.0);
    }
    let total = if changed {
        (1.08 * (impact + exploitability)).min(10.0)
    } else {
        (impact + exploitability).min(10.0)
    };
    Some(roundup(total))
}

fn roundup(value: f64) -> f64 {
    let scaled = (value * 100_000.0).round() as u64;
    if scaled.is_multiple_of(10_000) {
        scaled as f64 / 100_000.0
    } else {
        (scaled / 10_000 + 1) as f64 / 10.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_vectors_score_as_published() {
        for (vector, score) in [
            ("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H", 9.8),
            ("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:C/C:H/I:H/A:H", 10.0),
            ("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:N/A:H", 7.5),
            ("CVSS:3.1/AV:N/AC:L/PR:N/UI:R/S:C/C:L/I:L/A:N", 6.1),
            ("CVSS:3.0/AV:N/AC:L/PR:N/UI:R/S:U/C:L/I:L/A:N", 5.4),
        ] {
            let found = base_score(vector).unwrap();
            assert!((found - score).abs() < 1e-9, "{vector}: {found}");
        }
    }

    #[test]
    fn no_impact_scores_zero() {
        let score = base_score("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:N/A:N");
        assert_eq!(score, Some(0.0));
        assert_eq!(Severity::from_score(0.0), Severity::Unknown);
    }

    #[test]
    fn malformed_vectors_have_no_score() {
        for vector in [
            "",
            "CVSS:4.0/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H",
            "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H",
            "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H/A:H",
            "CVSS:3.1/AV:X/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H",
            "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H/E:Q",
            "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H/Z:P",
            "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H/E:P/E:F",
            "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H/RC:Z",
            "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H/MAV:H",
            "AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H",
            "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A",
            "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H/",
        ] {
            assert_eq!(base_score(vector), None, "{vector}");
        }
    }

    #[test]
    fn temporal_and_environmental_metrics_are_ignored_for_the_score() {
        assert_eq!(
            base_score("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H/E:P/RL:O/RC:C"),
            base_score("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H")
        );
        let score = base_score("CVSS:3.1/RC:C/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H/MAV:P/CR:X");
        assert!(score.is_some_and(|found| (found - 9.8).abs() < 1e-9));
    }

    #[test]
    fn severity_bands_follow_d8() {
        for (score, label) in [
            (10.0, "critical"),
            (9.0, "critical"),
            (8.9, "high"),
            (7.0, "high"),
            (6.9, "medium"),
            (4.0, "medium"),
            (3.9, "low"),
            (0.1, "low"),
            (0.0, "unknown"),
        ] {
            assert_eq!(Severity::from_score(score).as_str(), label, "{score}");
        }
    }
}
