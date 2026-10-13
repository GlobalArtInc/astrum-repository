use std::fmt;

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

const MONTHS: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];
const WEEKDAYS: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
const FIRST_YEAR: i32 = 1970;
const LAST_YEAR: i32 = 2099;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct CronError(String);

fn invalid(message: impl Into<String>) -> CronError {
    CronError(message.into())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Bits(u128);

impl Bits {
    fn contains(&self, value: u32) -> bool {
        value < 128 && self.0 & (1u128 << value) != 0
    }

    fn insert(&mut self, value: u32) {
        self.0 |= 1u128 << value;
    }

    fn first_from(&self, start: u32, max: u32) -> Option<u32> {
        (start..=max).find(|value| self.contains(*value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DayOfMonth {
    Any,
    Days(Bits),
    Last { offset: u32 },
    Weekday(u32),
    LastWeekday,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DayOfWeek {
    Any,
    Days(Bits),
    Last(u32),
    Nth { day: u32, nth: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cron {
    source: String,
    seconds: Bits,
    minutes: Bits,
    hours: Bits,
    day_of_month: DayOfMonth,
    months: Bits,
    day_of_week: DayOfWeek,
    years: Option<Vec<i32>>,
}

impl fmt::Display for Cron {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.source)
    }
}

struct Field {
    name: &'static str,
    min: u32,
    max: u32,
    names: &'static [&'static str],
    first_name: u32,
}

const SECONDS: Field = Field {
    name: "seconds",
    min: 0,
    max: 59,
    names: &[],
    first_name: 0,
};
const MINUTES: Field = Field {
    name: "minutes",
    min: 0,
    max: 59,
    names: &[],
    first_name: 0,
};
const HOURS: Field = Field {
    name: "hours",
    min: 0,
    max: 23,
    names: &[],
    first_name: 0,
};
const DAYS: Field = Field {
    name: "day of the month",
    min: 1,
    max: 31,
    names: &[],
    first_name: 0,
};
const MONTH: Field = Field {
    name: "month",
    min: 1,
    max: 12,
    names: &MONTHS,
    first_name: 1,
};
const WEEKDAY: Field = Field {
    name: "day of the week",
    min: 1,
    max: 7,
    names: &WEEKDAYS,
    first_name: 1,
};
const YEAR: Field = Field {
    name: "year",
    min: FIRST_YEAR as u32,
    max: LAST_YEAR as u32,
    names: &[],
    first_name: 0,
};

impl Field {
    fn value(&self, text: &str) -> Result<u32, CronError> {
        let upper = text.to_ascii_uppercase();
        if let Some(index) = self.names.iter().position(|name| *name == upper) {
            return Ok(index as u32 + self.first_name);
        }
        let value: u32 = text
            .parse()
            .map_err(|_| invalid(format!("'{text}' is not a valid {}", self.name)))?;
        if value < self.min || value > self.max {
            return Err(invalid(format!(
                "the {} must be between {} and {}, not {value}",
                self.name, self.min, self.max
            )));
        }
        Ok(value)
    }

    fn values(&self, text: &str) -> Result<Vec<u32>, CronError> {
        let mut values = Vec::new();
        for item in text.split(',') {
            if item.is_empty() {
                return Err(invalid(format!("the {} has an empty item", self.name)));
            }
            let (range, step) = match item.split_once('/') {
                Some((range, step)) => {
                    let step: u32 = step.parse().map_err(|_| {
                        invalid(format!(
                            "'{step}' is not a valid step for the {}",
                            self.name
                        ))
                    })?;
                    if step == 0 || step > self.max - self.min + 1 {
                        return Err(invalid(format!(
                            "the step {step} does not fit the {}",
                            self.name
                        )));
                    }
                    (range, step)
                }
                None => (item, 1),
            };
            let (start, end) = match range {
                "*" => (self.min, self.max),
                range => match range.split_once('-') {
                    Some((start, end)) => (self.value(start)?, self.value(end)?),
                    None => {
                        let start = self.value(range)?;
                        match item.contains('/') {
                            true => (start, self.max),
                            false => (start, start),
                        }
                    }
                },
            };
            let span = self.max - self.min + 1;
            let length = match end >= start {
                true => end - start,
                false => end + span - start,
            };
            let mut offset = 0;
            while offset <= length {
                let value = start + offset;
                values.push(match value > self.max {
                    true => value - span,
                    false => value,
                });
                offset += step;
            }
        }
        Ok(values)
    }

    fn bits(&self, text: &str) -> Result<Bits, CronError> {
        let mut bits = Bits(0);
        for value in self.values(text)? {
            bits.insert(value);
        }
        Ok(bits)
    }
}

fn day_of_month(text: &str) -> Result<DayOfMonth, CronError> {
    let upper = text.to_ascii_uppercase();
    if upper == "?" {
        return Ok(DayOfMonth::Any);
    }
    if upper == "LW" {
        return Ok(DayOfMonth::LastWeekday);
    }
    if let Some(rest) = upper.strip_prefix('L') {
        let offset = match rest.strip_prefix('-') {
            Some(offset) => offset
                .parse::<u32>()
                .ok()
                .filter(|offset| *offset <= 30)
                .ok_or_else(|| invalid(format!("'{text}' is not a valid day of the month")))?,
            None if rest.is_empty() => 0,
            None => return Err(invalid(format!("'{text}' is not a valid day of the month"))),
        };
        return Ok(DayOfMonth::Last { offset });
    }
    if let Some(day) = upper.strip_suffix('W') {
        return Ok(DayOfMonth::Weekday(DAYS.value(day)?));
    }
    Ok(DayOfMonth::Days(DAYS.bits(text)?))
}

fn day_of_week(text: &str) -> Result<DayOfWeek, CronError> {
    let upper = text.to_ascii_uppercase();
    if upper == "?" {
        return Ok(DayOfWeek::Any);
    }
    if upper == "L" {
        return Ok(DayOfWeek::Days(WEEKDAY.bits("7")?));
    }
    if let Some(day) = upper.strip_suffix('L') {
        return Ok(DayOfWeek::Last(WEEKDAY.value(day)?));
    }
    if let Some((day, nth)) = upper.split_once('#') {
        let nth = nth
            .parse::<u32>()
            .ok()
            .filter(|nth| (1..=5).contains(nth))
            .ok_or_else(|| invalid(format!("'{text}' names no week between 1 and 5")))?;
        return Ok(DayOfWeek::Nth {
            day: WEEKDAY.value(day)?,
            nth,
        });
    }
    Ok(DayOfWeek::Days(WEEKDAY.bits(text)?))
}

impl Cron {
    pub fn parse(source: &str) -> Result<Self, CronError> {
        let fields: Vec<&str> = source.split_whitespace().collect();
        if fields.len() != 6 && fields.len() != 7 {
            return Err(invalid(format!(
                "'{source}' has {} fields; a cron expression has six or seven: \
                 seconds, minutes, hours, day of the month, month, day of the week and an optional year",
                fields.len()
            )));
        }
        let day_of_month = day_of_month(fields[3])?;
        let day_of_week = day_of_week(fields[5])?;
        match (&day_of_month, &day_of_week) {
            (DayOfMonth::Any, DayOfWeek::Any) => {
                return Err(invalid(
                    "'?' may stand for the day of the month or the day of the week, not both",
                ));
            }
            (DayOfMonth::Any, _) | (_, DayOfWeek::Any) => {}
            _ => {
                return Err(invalid(
                    "give either the day of the month or the day of the week and put '?' in the other",
                ));
            }
        }
        let years = match fields.get(6) {
            None | Some(&"*") => None,
            Some(text) => Some(
                YEAR.values(text)?
                    .into_iter()
                    .map(|year| year as i32)
                    .collect(),
            ),
        };
        Ok(Self {
            source: fields.join(" "),
            seconds: SECONDS.bits(fields[0])?,
            minutes: MINUTES.bits(fields[1])?,
            hours: HOURS.bits(fields[2])?,
            day_of_month,
            months: MONTH.bits(fields[4])?,
            day_of_week,
            years,
        })
    }

    pub fn next_after(&self, after: NaiveDateTime) -> Option<NaiveDateTime> {
        let start = after.with_nanosecond(0)? + Duration::seconds(1);
        let mut date = start.date();
        let mut first_day = true;
        while date.year() <= LAST_YEAR {
            if self.matches_day(date) {
                let from = match first_day {
                    true => start.time(),
                    false => NaiveTime::MIN,
                };
                if let Some(time) = self.first_time(from) {
                    return Some(date.and_time(time));
                }
            }
            date = self.next_candidate(date)?;
            first_day = false;
        }
        None
    }

    fn next_candidate(&self, date: NaiveDate) -> Option<NaiveDate> {
        let next = date.succ_opt()?;
        if let Some(years) = &self.years
            && !years.contains(&next.year())
        {
            let year = years
                .iter()
                .copied()
                .filter(|year| *year > next.year())
                .min()?;
            return NaiveDate::from_ymd_opt(year, 1, 1);
        }
        if !self.months.contains(next.month()) {
            let mut month = next.month();
            let mut year = next.year();
            loop {
                month += 1;
                if month > 12 {
                    month = 1;
                    year += 1;
                }
                if year > LAST_YEAR + 1 {
                    return None;
                }
                if self.months.contains(month) {
                    return NaiveDate::from_ymd_opt(year, month, 1);
                }
            }
        }
        Some(next)
    }

    fn first_time(&self, from: NaiveTime) -> Option<NaiveTime> {
        for hour in from.hour()..24 {
            if !self.hours.contains(hour) {
                continue;
            }
            let first_minute = if hour == from.hour() {
                from.minute()
            } else {
                0
            };
            for minute in first_minute..60 {
                if !self.minutes.contains(minute) {
                    continue;
                }
                let first_second = if hour == from.hour() && minute == from.minute() {
                    from.second()
                } else {
                    0
                };
                if let Some(second) = self.seconds.first_from(first_second, 59) {
                    return NaiveTime::from_hms_opt(hour, minute, second);
                }
            }
        }
        None
    }

    fn matches_day(&self, date: NaiveDate) -> bool {
        if self
            .years
            .as_ref()
            .is_some_and(|years| !years.contains(&date.year()))
        {
            return false;
        }
        if !self.months.contains(date.month()) {
            return false;
        }
        let last = last_day(date);
        let weekday = date.weekday().number_from_sunday();
        let day_of_month = match &self.day_of_month {
            DayOfMonth::Any => true,
            DayOfMonth::Days(days) => days.contains(date.day()),
            DayOfMonth::Last { offset } => last.checked_sub(*offset) == Some(date.day()),
            DayOfMonth::Weekday(day) => nearest_weekday(date, (*day).min(last)) == date.day(),
            DayOfMonth::LastWeekday => nearest_weekday(date, last) == date.day(),
        };
        let day_of_week = match &self.day_of_week {
            DayOfWeek::Any => true,
            DayOfWeek::Days(days) => days.contains(weekday),
            DayOfWeek::Last(day) => *day == weekday && date.day() + 7 > last,
            DayOfWeek::Nth { day, nth } => *day == weekday && (date.day() - 1) / 7 + 1 == *nth,
        };
        day_of_month && day_of_week
    }
}

fn last_day(date: NaiveDate) -> u32 {
    let (year, month) = match date.month() {
        12 => (date.year() + 1, 1),
        month => (date.year(), month + 1),
    };
    NaiveDate::from_ymd_opt(year, month, 1)
        .and_then(|first| first.pred_opt())
        .map(|last| last.day())
        .unwrap_or(28)
}

fn nearest_weekday(date: NaiveDate, day: u32) -> u32 {
    let Some(target) = date.with_day(day) else {
        return 0;
    };
    let last = last_day(date);
    match target.weekday().number_from_sunday() {
        7 if day == 1 => 3,
        7 => day - 1,
        1 if day == last => day - 2,
        1 => day + 1,
        _ => day,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    fn next(expression: &str, after: &str) -> String {
        Cron::parse(expression)
            .unwrap_or_else(|error| panic!("{expression}: {error}"))
            .next_after(at(after))
            .map(|time| time.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "never".to_owned())
    }

    #[test]
    fn expressions_fire_when_the_reference_server_said_they_would() {
        let now = "2026-10-12 19:32:22";
        assert_eq!(next("0 0 1 * * ?", now), "2026-10-13 01:00:00");
        assert_eq!(next("0 15 10 ? * MON-FRI", now), "2026-10-13 10:15:00");
        assert_eq!(next("0 0 1 L * ?", now), "2026-10-31 01:00:00");
        assert_eq!(next("0 0 1 ? * 6#3", now), "2026-10-16 01:00:00");
        assert_eq!(next("0 0 1 15W * ?", now), "2026-10-15 01:00:00");
        assert_eq!(next("0 0 1 ? * 5L", now), "2026-10-29 01:00:00");
        assert_eq!(next("0/30 * * * * ?", now), "2026-10-12 19:32:30");
        assert_eq!(next("0 0 12 ? * SUN", now), "2026-10-18 12:00:00");
        assert_eq!(next("0 0 12 ? * 1", now), "2026-10-18 12:00:00");
        assert_eq!(
            next("0 0 12 ? JAN-MAR 2-6 2030", now),
            "2030-01-01 12:00:00"
        );
        assert_eq!(
            next("0 */30 * * * ?", "2026-10-12 19:30:00"),
            "2026-10-12 20:00:00"
        );
        assert_eq!(next("0 0 1 * * ? *", now), "2026-10-13 01:00:00");
    }

    #[test]
    fn day_rules_follow_the_calendar() {
        assert_eq!(
            next("0 0 0 LW * ?", "2026-10-12 00:00:00"),
            "2026-10-30 00:00:00"
        );
        assert_eq!(
            next("0 0 0 L-2 * ?", "2026-02-01 00:00:00"),
            "2026-02-26 00:00:00"
        );
        assert_eq!(
            next("0 0 0 1W * ?", "2026-10-30 00:00:00"),
            "2026-11-02 00:00:00"
        );
        assert_eq!(
            next("0 0 0 29 2 ?", "2026-01-01 00:00:00"),
            "2028-02-29 00:00:00"
        );
        assert_eq!(
            next("0 0 22-2 * * ?", "2026-10-12 23:30:00"),
            "2026-10-13 00:00:00"
        );
        assert_eq!(
            next("0 0 0 ? * FRI-MON", "2026-10-13 00:00:00"),
            "2026-10-16 00:00:00"
        );
        assert_eq!(
            next("0 0 0 ? * L", "2026-10-12 00:00:00"),
            "2026-10-17 00:00:00"
        );
        assert_eq!(next("0 0 12 ? * 2-6 2020", "2026-10-12 00:00:00"), "never");
        assert_eq!(next("0 0 0 30 2 ?", "2026-10-12 00:00:00"), "never");
    }

    #[test]
    fn malformed_expressions_are_refused() {
        for bad in [
            "",
            "0 15 10 * *",
            "bogus",
            "0 0 1 * * *",
            "0 0 12 1 * 2",
            "0 0 1 ? * ?",
            "60 0 1 * * ?",
            "0 0 24 * * ?",
            "0 0 1 32 * ?",
            "0 0 1 * 13 ?",
            "0 0 1 ? * 8",
            "0 0 1 ? * 6#6",
            "0 0/0 1 * * ?",
            "0 0 1 * * ? 1969",
            "0 0 1 ,1 * ?",
            "0 0 1 ? FOO *",
        ] {
            assert!(Cron::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_source_is_kept_with_single_spaces() {
        assert_eq!(
            Cron::parse("0  0 1 * *  ?").unwrap().to_string(),
            "0 0 1 * * ?"
        );
    }
}
