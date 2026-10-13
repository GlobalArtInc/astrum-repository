use chrono::{
    DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime, SecondsFormat, TimeZone,
    Utc,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::Problems;
use crate::cron::Cron;

pub const LAST_DAY_OF_MONTH: u32 = 999;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Manual,
    Once,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Cron,
}

impl Kind {
    pub fn parse(text: &str) -> Option<Self> {
        Some(match text.trim().to_ascii_lowercase().as_str() {
            "manual" => Kind::Manual,
            "once" => Kind::Once,
            "hourly" => Kind::Hourly,
            "daily" => Kind::Daily,
            "weekly" => Kind::Weekly,
            "monthly" => Kind::Monthly,
            "cron" | "advanced" => Kind::Cron,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Manual => "manual",
            Kind::Once => "once",
            Kind::Hourly => "hourly",
            Kind::Daily => "daily",
            Kind::Weekly => "weekly",
            Kind::Monthly => "monthly",
            Kind::Cron => "cron",
        }
    }

    pub fn public_name(self) -> &'static str {
        match self {
            Kind::Cron => "advanced",
            other => other.as_str(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrequencyRequest {
    pub schedule: Option<String>,
    pub start_date: Option<i64>,
    pub time_zone_offset: Option<String>,
    pub recurring_days: Option<Vec<i64>>,
    pub cron_expression: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schedule {
    pub kind: Kind,
    pub start: Option<DateTime<Utc>>,
    pub offset: Option<FixedOffset>,
    pub days: Vec<u32>,
    pub cron: Option<Cron>,
}

impl Schedule {
    pub fn manual() -> Self {
        Self {
            kind: Kind::Manual,
            start: None,
            offset: None,
            days: Vec::new(),
            cron: None,
        }
    }

    pub fn cron(expression: &str) -> Result<Self, String> {
        Ok(Self {
            kind: Kind::Cron,
            start: Some(Utc::now()),
            offset: None,
            days: Vec::new(),
            cron: Some(Cron::parse(expression).map_err(|error| error.to_string())?),
        })
    }

    pub fn hourly(start: DateTime<Utc>) -> Self {
        Self {
            kind: Kind::Hourly,
            start: Some(start),
            ..Self::manual()
        }
    }

    pub fn from_request(
        request: &FrequencyRequest,
        now: DateTime<Utc>,
        problems: &mut Problems,
    ) -> Option<Self> {
        let Some(name) = request
            .schedule
            .as_deref()
            .filter(|name| !name.trim().is_empty())
        else {
            problems.add("frequency.schedule", "say how the task is scheduled");
            return None;
        };
        let Some(kind) = Kind::parse(name) else {
            problems.add(
                "frequency.schedule",
                format!(
                    "'{name}' is not a schedule; use manual, once, hourly, daily, weekly, monthly or cron"
                ),
            );
            return None;
        };
        let offset = match request.time_zone_offset.as_deref() {
            None => None,
            Some(text) if text.trim().is_empty() => None,
            Some(text) => match parse_offset(text) {
                Some(offset) => Some(offset),
                None => {
                    problems.add(
                        "frequency.timeZoneOffset",
                        format!("'{text}' is not a time zone offset such as +03:00 or Z"),
                    );
                    return None;
                }
            },
        };
        let mut schedule = Self {
            kind,
            offset,
            ..Self::manual()
        };
        match kind {
            Kind::Manual => {
                schedule.offset = None;
                return Some(schedule);
            }
            Kind::Cron => {
                let Some(expression) = request
                    .cron_expression
                    .as_deref()
                    .filter(|text| !text.trim().is_empty())
                else {
                    problems.add(
                        "frequency.cronExpression",
                        "a cron schedule needs an expression",
                    );
                    return None;
                };
                match Cron::parse(expression) {
                    Ok(cron) => schedule.cron = Some(cron),
                    Err(error) => {
                        problems.add(
                            "frequency.cronExpression",
                            format!("'{expression}' is not a valid cron expression: {error}"),
                        );
                        return None;
                    }
                }
                schedule.start = Some(now);
            }
            _ => {
                let Some(millis) = request.start_date else {
                    problems.add("frequency.startDate", "this schedule needs a start date");
                    return None;
                };
                let Some(start) = Utc.timestamp_millis_opt(millis).single() else {
                    problems.add("frequency.startDate", "the start date is out of range");
                    return None;
                };
                let start = start
                    .with_timezone(&Utc)
                    .date_naive()
                    .and_hms_opt(
                        chrono::Timelike::hour(&start),
                        chrono::Timelike::minute(&start),
                        0,
                    )
                    .map(|naive| naive.and_utc())
                    .unwrap_or(start);
                if kind == Kind::Once && start <= now {
                    problems.add(
                        "frequency.startDate",
                        "the start date must be in the future",
                    );
                    return None;
                }
                schedule.start = Some(start);
            }
        }
        if matches!(kind, Kind::Weekly | Kind::Monthly) {
            let days = request.recurring_days.clone().unwrap_or_default();
            if days.is_empty() {
                problems.add("frequency.recurringDays", "choose at least one day");
                return None;
            }
            let mut checked = Vec::new();
            for day in days {
                let valid = match kind {
                    Kind::Weekly => (1..=7).contains(&day),
                    _ => (1..=31).contains(&day) || day == i64::from(LAST_DAY_OF_MONTH),
                };
                if !valid {
                    let what = match kind {
                        Kind::Weekly => "a day of the week (1 is Sunday, 7 is Saturday)",
                        _ => "a day of the month (1 to 31, or 999 for the last day)",
                    };
                    problems.add("frequency.recurringDays", format!("{day} is not {what}"));
                    return None;
                }
                checked.push(day as u32);
            }
            checked.sort_unstable();
            checked.dedup();
            schedule.days = checked;
        }
        if schedule.kind != Kind::Once && schedule.next_after(now).is_none() {
            problems.add("frequency", "this schedule never runs the task");
            return None;
        }
        Some(schedule)
    }

    pub fn next_after(&self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let offset = self
            .offset
            .unwrap_or_else(|| FixedOffset::east_opt(0).expect("UTC"));
        match self.kind {
            Kind::Manual => None,
            Kind::Once => self.start.filter(|start| *start > now),
            Kind::Hourly => step(self.start?, now, Duration::hours(1)),
            Kind::Daily => step(self.start?, now, Duration::days(1)),
            Kind::Weekly | Kind::Monthly => {
                let start = self.start?.with_timezone(&offset);
                let time = start.time();
                let local_now = now.with_timezone(&offset).naive_local();
                let mut date = start
                    .date_naive()
                    .max(local_now.date().pred_opt().unwrap_or(local_now.date()));
                for _ in 0..(366 * 5) {
                    if self.day_matches(date) {
                        let candidate = date.and_time(time);
                        if candidate > local_now {
                            return local(offset, candidate);
                        }
                    }
                    date = date.succ_opt()?;
                }
                None
            }
            Kind::Cron => {
                let cron = self.cron.as_ref()?;
                let local_now = now.with_timezone(&offset).naive_local();
                cron.next_after(local_now)
                    .and_then(|next| local(offset, next))
            }
        }
    }

    fn day_matches(&self, date: NaiveDate) -> bool {
        match self.kind {
            Kind::Weekly => self.days.contains(&date.weekday().number_from_sunday()),
            Kind::Monthly => {
                self.days.contains(&date.day())
                    || (self.days.contains(&LAST_DAY_OF_MONTH)
                        && date
                            .succ_opt()
                            .is_some_and(|next| next.month() != date.month()))
            }
            _ => false,
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "schedule": self.kind.as_str(),
            "startDate": self.start.map(|start| start.timestamp_millis()),
            "timeZoneOffset": self.offset.map(format_offset),
            "recurringDays": match self.days.is_empty() {
                true => Value::Null,
                false => json!(self.days),
            },
            "cronExpression": self.cron.as_ref().map(ToString::to_string),
        })
    }

    pub fn from_json(value: &Value) -> Self {
        let request: FrequencyRequest = serde_json::from_value(value.clone()).unwrap_or_default();
        let kind = request
            .schedule
            .as_deref()
            .and_then(Kind::parse)
            .unwrap_or(Kind::Manual);
        Self {
            kind,
            start: request
                .start_date
                .and_then(|millis| Utc.timestamp_millis_opt(millis).single()),
            offset: request.time_zone_offset.as_deref().and_then(parse_offset),
            days: request
                .recurring_days
                .unwrap_or_default()
                .into_iter()
                .filter_map(|day| u32::try_from(day).ok())
                .collect(),
            cron: request
                .cron_expression
                .as_deref()
                .and_then(|expression| Cron::parse(expression).ok()),
        }
    }

    pub fn public_offset(&self) -> Option<String> {
        match (self.offset, self.kind) {
            (Some(offset), _) => Some(format_offset(offset)),
            (None, Kind::Cron) => Some("Z".to_owned()),
            (None, _) => None,
        }
    }
}

fn step(start: DateTime<Utc>, now: DateTime<Utc>, every: Duration) -> Option<DateTime<Utc>> {
    if start > now {
        return Some(start);
    }
    let passed = (now - start).num_seconds() / every.num_seconds() + 1;
    Some(start + every * passed as i32)
}

fn local(offset: FixedOffset, naive: NaiveDateTime) -> Option<DateTime<Utc>> {
    offset
        .from_local_datetime(&naive)
        .single()
        .map(|time| time.with_timezone(&Utc))
}

pub fn parse_offset(text: &str) -> Option<FixedOffset> {
    let text = text.trim();
    if text.eq_ignore_ascii_case("z") || text.eq_ignore_ascii_case("utc") {
        return FixedOffset::east_opt(0);
    }
    let (sign, rest) = match text.as_bytes().first()? {
        b'+' => (1, &text[1..]),
        b'-' => (-1, &text[1..]),
        _ => return None,
    };
    let (hours, minutes) = match rest.split_once(':') {
        Some((hours, minutes)) => (hours, minutes),
        None if rest.len() == 4 => (&rest[..2], &rest[2..]),
        None => (rest, "0"),
    };
    let hours: i32 = hours.parse().ok().filter(|hours| *hours <= 18)?;
    let minutes: i32 = minutes.parse().ok().filter(|minutes| *minutes < 60)?;
    FixedOffset::east_opt(sign * (hours * 3600 + minutes * 60))
}

pub fn format_offset(offset: FixedOffset) -> String {
    let seconds = offset.local_minus_utc();
    if seconds == 0 {
        return "Z".to_owned();
    }
    let sign = if seconds < 0 { '-' } else { '+' };
    let seconds = seconds.abs();
    format!("{sign}{:02}:{:02}", seconds / 3600, (seconds % 3600) / 60)
}

pub fn public_time(time: DateTime<Utc>) -> String {
    let text = time.to_rfc3339_opts(SecondsFormat::Millis, true);
    match text.strip_suffix('Z') {
        Some(rest) => format!("{rest}+00:00"),
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-12T19:31:39Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn schedule(request: Value) -> Result<Schedule, Vec<String>> {
        let request: FrequencyRequest = serde_json::from_value(request).unwrap();
        let mut problems = Problems::default();
        let schedule = Schedule::from_request(&request, now(), &mut problems);
        match schedule {
            Some(schedule) => Ok(schedule),
            None => Err(problems
                .into_inner()
                .into_iter()
                .map(|problem| problem.id)
                .collect()),
        }
    }

    fn next(request: Value) -> Option<String> {
        schedule(request)
            .unwrap()
            .next_after(now())
            .map(public_time)
    }

    #[test]
    fn next_runs_match_the_reference_server() {
        assert_eq!(
            next(json!({"schedule": "hourly", "startDate": 1791886500})),
            Some("2026-10-12T19:44:00.000+00:00".to_owned())
        );
        assert_eq!(
            next(json!({"schedule": "daily", "startDate": 1791886500})),
            Some("2026-10-13T17:44:00.000+00:00".to_owned())
        );
        assert_eq!(
            next(json!({"schedule": "weekly", "startDate": 1791886500, "recurringDays": [1, 3]})),
            Some("2026-10-13T17:44:00.000+00:00".to_owned())
        );
        assert_eq!(
            next(
                json!({"schedule": "monthly", "startDate": 1791886500, "recurringDays": [1, 15, 31]})
            ),
            Some("2026-10-15T17:44:00.000+00:00".to_owned())
        );
        assert_eq!(
            next(
                json!({"schedule": "monthly", "startDate": 1791900000000_i64, "recurringDays": [999]})
            ),
            Some("2026-10-31T14:00:00.000+00:00".to_owned())
        );
        assert_eq!(
            next(
                json!({"schedule": "weekly", "startDate": 1791900000000_i64, "recurringDays": [2]})
            ),
            Some("2026-10-19T14:00:00.000+00:00".to_owned())
        );
        assert_eq!(
            next(
                json!({"schedule": "weekly", "startDate": 1791900000000_i64, "recurringDays": [7]})
            ),
            Some("2026-10-17T14:00:00.000+00:00".to_owned())
        );
        assert_eq!(
            next(json!({"schedule": "once", "startDate": 1791900000000_i64})),
            Some("2026-10-13T14:00:00.000+00:00".to_owned())
        );
        assert_eq!(
            next(json!({"schedule": "cron", "cronExpression": "0 15 10 ? * MON-FRI"})),
            Some("2026-10-13T10:15:00.000+00:00".to_owned())
        );
        assert_eq!(next(json!({"schedule": "manual", "startDate": 1})), None);
    }

    #[test]
    fn offsets_decide_the_local_day() {
        let request = json!({
            "schedule": "weekly",
            "startDate": DateTime::parse_from_rfc3339("2026-10-12T22:30:00Z").unwrap().timestamp_millis(),
            "timeZoneOffset": "+03:00",
            "recurringDays": [3]
        });
        assert_eq!(
            next(request),
            Some("2026-10-12T22:30:00.000+00:00".to_owned())
        );
        let cron = json!({"schedule": "cron", "cronExpression": "0 0 1 * * ?", "timeZoneOffset": "+03:00"});
        assert_eq!(next(cron), Some("2026-10-12T22:00:00.000+00:00".to_owned()));
    }

    #[test]
    fn bad_frequencies_name_their_field() {
        for (request, field) in [
            (json!({"schedule": "weird"}), "frequency.schedule"),
            (json!({}), "frequency.schedule"),
            (json!({"schedule": "daily"}), "frequency.startDate"),
            (
                json!({"schedule": "once", "startDate": 1791886500}),
                "frequency.startDate",
            ),
            (
                json!({"schedule": "weekly", "startDate": 1791886500}),
                "frequency.recurringDays",
            ),
            (
                json!({"schedule": "weekly", "startDate": 1, "recurringDays": [0, 8]}),
                "frequency.recurringDays",
            ),
            (
                json!({"schedule": "monthly", "startDate": 1, "recurringDays": [32]}),
                "frequency.recurringDays",
            ),
            (json!({"schedule": "cron"}), "frequency.cronExpression"),
            (
                json!({"schedule": "cron", "cronExpression": "0 15 10 * *"}),
                "frequency.cronExpression",
            ),
            (
                json!({"schedule": "cron", "cronExpression": "0 0 12 ? * 2-6 2020"}),
                "frequency",
            ),
            (
                json!({"schedule": "daily", "startDate": 1, "timeZoneOffset": "Mars"}),
                "frequency.timeZoneOffset",
            ),
        ] {
            assert_eq!(
                schedule(request.clone()),
                Err(vec![field.to_owned()]),
                "{request}"
            );
        }
    }

    #[test]
    fn schedules_survive_storage() {
        for request in [
            json!({"schedule": "manual"}),
            json!({"schedule": "weekly", "startDate": 1791900000000_i64, "recurringDays": [2, 4], "timeZoneOffset": "+03:00"}),
            json!({"schedule": "cron", "cronExpression": "0 0 1 * * ?"}),
        ] {
            let schedule = schedule(request).unwrap();
            assert_eq!(Schedule::from_json(&schedule.to_json()), schedule);
        }
    }

    #[test]
    fn offsets_parse_and_print() {
        assert_eq!(format_offset(parse_offset("+03:00").unwrap()), "+03:00");
        assert_eq!(format_offset(parse_offset("-0530").unwrap()), "-05:30");
        assert_eq!(format_offset(parse_offset("Z").unwrap()), "Z");
        assert!(parse_offset("+25:00").is_none());
        assert!(parse_offset("03:00").is_none());
    }
}
