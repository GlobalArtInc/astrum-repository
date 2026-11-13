use astrum_blob::QuotaKind;
use astrum_blob::config::StoreConfig;
use astrum_db::entity::blob_store_usage_day;
use chrono::NaiveDate;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use serde::Serialize;

use crate::routes::blobstores::Measured;

pub const WINDOW_DAYS: u64 = 30;
pub const MINIMUM_POINTS: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Basis {
    Capacity,
    SoftQuota,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Headroom {
    pub basis: Basis,
    pub bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Forecast {
    pub days_until_full: Option<f64>,
    pub bytes_per_day: Option<f64>,
    pub basis: Option<Basis>,
    pub sample_days: usize,
}

pub fn slope(points: &[(NaiveDate, i64)]) -> Option<f64> {
    if points.len() < MINIMUM_POINTS {
        return None;
    }
    let first_day = points.iter().map(|(day, _)| *day).min()?;
    let first_bytes = points
        .iter()
        .find(|(day, _)| *day == first_day)
        .map(|(_, bytes)| *bytes)?;
    let samples: Vec<(f64, f64)> = points
        .iter()
        .map(|(day, bytes)| {
            (
                (*day - first_day).num_days() as f64,
                (i128::from(*bytes) - i128::from(first_bytes)) as f64,
            )
        })
        .collect();
    let count = samples.len() as f64;
    let mean_x = samples.iter().map(|(x, _)| x).sum::<f64>() / count;
    let mean_y = samples.iter().map(|(_, y)| y).sum::<f64>() / count;
    let spread: f64 = samples.iter().map(|(x, _)| (x - mean_x).powi(2)).sum();
    if spread == 0.0 {
        return None;
    }
    let covariance: f64 = samples
        .iter()
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum();
    Some(covariance / spread)
}

pub fn forecast(points: &[(NaiveDate, i64)], headroom: Option<Headroom>) -> Forecast {
    let bytes_per_day = slope(points);
    let days_until_full = match (bytes_per_day, headroom) {
        (Some(slope), Some(headroom)) if slope > 0.0 => {
            Some((headroom.bytes as f64 / slope * 10.0).round() / 10.0)
        }
        _ => None,
    };
    Forecast {
        days_until_full,
        bytes_per_day: bytes_per_day.map(f64::round),
        basis: headroom.map(|headroom| headroom.basis),
        sample_days: points.len(),
    }
}

pub(crate) fn headroom(config: &StoreConfig, measured: &Measured) -> Option<Headroom> {
    if config.group().is_some() || measured.unavailable {
        return None;
    }
    let capacity = measured.available.map(|bytes| Headroom {
        basis: Basis::Capacity,
        bytes,
    });
    let quota = config
        .soft_quota
        .filter(|quota| quota.kind == QuotaKind::SpaceUsedQuota)
        .map(|quota| Headroom {
            basis: Basis::SoftQuota,
            bytes: quota.limit.saturating_sub(measured.totals.total_bytes),
        });
    match (capacity, quota) {
        (Some(capacity), Some(quota)) => Some(match quota.bytes < capacity.bytes {
            true => quota,
            false => capacity,
        }),
        (capacity, quota) => capacity.or(quota),
    }
}

pub async fn series(
    db: &impl ConnectionTrait,
    store: &str,
) -> Result<Vec<(NaiveDate, i64)>, DbErr> {
    let mut rows = blob_store_usage_day::Entity::find()
        .filter(blob_store_usage_day::Column::Store.eq(store))
        .order_by_desc(blob_store_usage_day::Column::Day)
        .limit(WINDOW_DAYS)
        .all(db)
        .await?;
    rows.reverse();
    Ok(rows
        .into_iter()
        .map(|row| (row.day, row.total_bytes))
        .collect())
}

pub(crate) async fn of_store(
    db: &impl ConnectionTrait,
    config: &StoreConfig,
    measured: &Measured,
) -> Result<Forecast, DbErr> {
    if config.group().is_some() {
        return Ok(forecast(&[], None));
    }
    let points = series(db, &config.name).await?;
    Ok(forecast(&points, headroom(config, measured)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(offset: i64) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 1).unwrap() + chrono::Duration::days(offset)
    }

    fn series_of(values: &[i64]) -> Vec<(NaiveDate, i64)> {
        values
            .iter()
            .enumerate()
            .map(|(index, bytes)| (day(index as i64), *bytes))
            .collect()
    }

    fn room(bytes: u64) -> Option<Headroom> {
        Some(Headroom {
            basis: Basis::Capacity,
            bytes,
        })
    }

    #[test]
    fn a_flat_series_has_no_forecast() {
        let found = forecast(&series_of(&[500; 10]), room(1000));
        assert_eq!(found.bytes_per_day, Some(0.0));
        assert_eq!(found.days_until_full, None);
        assert_eq!(found.sample_days, 10);
    }

    #[test]
    fn a_growing_series_gives_the_days_left() {
        let values: Vec<i64> = (0..10).map(|index| 1000 + index * 100).collect();
        let found = forecast(&series_of(&values), room(5000));
        assert_eq!(found.bytes_per_day.map(f64::round), Some(100.0));
        assert_eq!(found.days_until_full.map(f64::round), Some(50.0));
        assert_eq!(found.basis, Some(Basis::Capacity));
    }

    #[test]
    fn a_shrinking_series_has_no_forecast() {
        let values: Vec<i64> = (0..10).map(|index| 5000 - index * 100).collect();
        let found = forecast(&series_of(&values), room(5000));
        assert!(found.bytes_per_day.unwrap() < 0.0);
        assert_eq!(found.days_until_full, None);
    }

    #[test]
    fn fewer_than_seven_points_have_no_forecast() {
        let found = forecast(&series_of(&[1, 2, 3, 4, 5, 6]), room(1000));
        assert_eq!(found.bytes_per_day, None);
        assert_eq!(found.days_until_full, None);
        assert_eq!(found.sample_days, 6);
        assert!(
            forecast(&series_of(&[1, 2, 3, 4, 5, 6, 7]), room(1000))
                .days_until_full
                .is_some()
        );
    }

    #[test]
    fn gaps_of_days_count_by_their_dates() {
        let offsets = [0, 2, 4, 6, 8, 10, 12];
        let points: Vec<(NaiveDate, i64)> = offsets
            .iter()
            .map(|offset| (day(*offset), 1000 + offset * 100))
            .collect();
        let found = forecast(&points, room(1000));
        assert_eq!(found.bytes_per_day.map(f64::round), Some(100.0));
        assert_eq!(found.days_until_full.map(f64::round), Some(10.0));
        assert_eq!(found.sample_days, 7);
    }

    #[test]
    fn without_headroom_there_is_a_slope_but_no_days() {
        let values: Vec<i64> = (0..10).map(|index| index * 100).collect();
        let found = forecast(&series_of(&values), None);
        assert!(found.bytes_per_day.is_some());
        assert_eq!(found.days_until_full, None);
        assert_eq!(found.basis, None);
    }
}
