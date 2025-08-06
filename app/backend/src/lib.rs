use chrono::{DateTime, Datelike, Days, DurationRound, Local, NaiveDate, TimeDelta};
use deadpool_redis::{Connection, Pool};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket::{
    form::FromForm,
    http::{Cookie, CookieJar},
    time::OffsetDateTime,
    State,
};
use sea_orm::sqlx::types::chrono::TimeZone;

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
    redis_pool.get().await
        .map_err(|e| format!("Failed to get redis connection: {:?}", e))
}

// ランダムなセッションIDを生成する関数
fn generate_session_id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30) // 30文字のランダムな文字列を生成
        .map(char::from)
        .collect()
}

// cookieの有効期限を取得
pub fn create_expiration() -> chrono::DateTime<Local> {
    // 現在の日付を取得
    let now = Local::now();

    // 今日の23:59:59を設定
    Local
        .with_ymd_and_hms(now.year(), now.month(), now.day() + 1, 0, 0, 0)
        .single()
        .expect("Failed to create expiration date time. The date should be valid.")
}

// 期限付きのsession_idをCookieに設定
pub fn create_session_cookie_with_expiration(
    cookies: &CookieJar<'_>,
    expiration: chrono::DateTime<Local>,
) -> String {
    let session_id = generate_session_id();

    // chronoのDateTimeをtime::OffsetDateTimeに変換
    let offset_expiration = OffsetDateTime::from_unix_timestamp(expiration.timestamp())
        .expect("Failed to convert timestamp to OffsetDateTime. The timestamp should be valid.");

    // Cookieを作成し、有効期限を設定
    let mut cookie = Cookie::new("session", session_id.clone());
    cookie.set_expires(offset_expiration);

    log::debug!("cookie value: {:?}", cookie.value());
    log::debug!("cookie expires:{:?}", cookie.expires());

    cookies.add(cookie);

    session_id
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
                start_date_input.year().into(),
                (start_date_input.month() as u32).into(),
                start_date_input.day().into(),
            )
            .expect("Failed to create NaiveDate from date components");
            start_date = Local
                .from_local_datetime(&naive_start_date.and_hms_opt(0, 0, 0)
                    .expect("Failed to create time with and_hms_opt"))
                .single()
                .expect("Failed to convert to Local datetime");
        }
        if let Some(end_date_input) = date_range.end_date {
            log::debug!("end_date_input: {:?}", end_date_input);
            let naive_end_date = NaiveDate::from_ymd_opt(
                end_date_input.year().into(),
                (end_date_input.month() as u32).into(),
                end_date_input.day().into(),
            )
            .expect("Failed to create NaiveDate from date components");
            end_date = Local
                .from_local_datetime(&naive_end_date.and_hms_opt(0, 0, 0)
                    .expect("Failed to create time with and_hms_opt"))
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
