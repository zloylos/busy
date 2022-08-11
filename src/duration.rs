use chrono::{Datelike, Timelike};

pub fn get_duration(period_days: i64) -> chrono::Duration {
  return chrono::Duration::days(period_days)
    .checked_add(&get_duration_from_midnight())
    .unwrap();
}

pub fn get_duration_from_week_start() -> chrono::Duration {
  return chrono::Duration::days(chrono::Local::now().weekday().num_days_from_monday() as i64)
    .checked_add(&get_duration_from_midnight())
    .unwrap();
}

pub fn get_duration_from_midnight() -> chrono::Duration {
  return chrono::Duration::seconds(
    chrono::Local::now()
      .time()
      .num_seconds_from_midnight()
      .into(),
  );
}
