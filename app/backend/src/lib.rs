pub mod config;
pub mod di;
pub mod handlers;
pub mod http;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod sea_generated;
pub mod services;

use chrono::{DateTime, Datelike, Days, DurationRound, Local, NaiveDate, TimeDelta, TimeZone};
use deadpool_redis::{Connection, Pool};
use rocket::{form::FromForm, State};

#[derive(FromForm)]
pub struct DateRange {
    start_date: Option<rocket::time::Date>,
    end_date: Option<rocket::time::Date>,
}

pub struct DateRangeLocal {
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
}

pub async fn get_redis_connection(redis_pool: &State<Pool>) -> Result<Connection, String> {
    redis_pool
        .get()
        .await
        .map_err(|e| format!("Failed to get redis connection: {e:?}"))
}


// date_rangeがNoneのときのデフォルト値を設定
pub fn get_date_range_local(date_range: Option<DateRange>) -> DateRangeLocal {
    // 期間の指定がない場合は、先週の月曜日から日曜日までの期間を取得
    let mut start_date: DateTime<Local> = Local::now()
        .duration_trunc(TimeDelta::days(1))
        .expect("Failed to truncate duration to days")
        .checked_sub_days(Days::new(
            (Local::now().weekday().number_from_monday() + 6) as u64,
        ))
        .expect("Failed to calculate date with checked_sub_days");
    let mut end_date: DateTime<Local> = Local::now()
        .duration_trunc(TimeDelta::days(1))
        .expect("Failed to truncate duration to days")
        .checked_sub_days(Days::new(
            (Local::now().weekday().number_from_monday() - 1) as u64,
        ))
        .expect("Failed to calculate date with checked_sub_days");
    if let Some(date_range) = date_range {
        if let Some(start_date_input) = date_range.start_date {
            log::debug!("start_date_input: {:?}", start_date_input);
            let naive_start_date = NaiveDate::from_ymd_opt(
                start_date_input.year(),
                start_date_input.month() as u32,
                start_date_input.day().into(),
            )
            .expect("Failed to create NaiveDate from date components");
            start_date = Local
                .from_local_datetime(
                    &naive_start_date
                        .and_hms_opt(0, 0, 0)
                        .expect("Failed to create time with and_hms_opt"),
                )
                .single()
                .expect("Failed to convert to Local datetime");
        }
        if let Some(end_date_input) = date_range.end_date {
            log::debug!("end_date_input: {:?}", end_date_input);
            let naive_end_date = NaiveDate::from_ymd_opt(
                end_date_input.year(),
                end_date_input.month() as u32,
                end_date_input.day().into(),
            )
            .expect("Failed to create NaiveDate from date components");
            end_date = Local
                .from_local_datetime(
                    &naive_end_date
                        .and_hms_opt(0, 0, 0)
                        .expect("Failed to create time with and_hms_opt"),
                )
                .single()
                .expect("Failed to convert to Local datetime");
        }
    };
    log::debug!("start_date: {:?}", start_date);
    log::debug!("end_date: {:?}", end_date);
    DateRangeLocal {
        start_date,
        end_date,
    }
}
