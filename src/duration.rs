use chrono::{Datelike, Timelike};

pub struct Period {
  pub from: chrono::DateTime<chrono::Local>,
  pub to: chrono::DateTime<chrono::Local>,
}

impl Period {
  pub fn new_to_now(from: chrono::DateTime<chrono::Local>) -> Self {
    return Self {
      from,
      to: chrono::Local::now(),
    };
  }

  pub fn contains(&self, moment: &chrono::DateTime<chrono::Local>) -> bool {
    return &self.from <= moment && moment <= &self.to;
  }
}

pub fn get_period_since_now(period_days: i64) -> chrono::DateTime<chrono::Local> {
  return get_checked_sub_signed_from_now(
    chrono::Duration::days(period_days)
      .checked_add(&get_duration_from_midnight())
      .unwrap(),
  );
}

pub fn get_week_start_datetime() -> chrono::DateTime<chrono::Local> {
  return get_checked_sub_signed_from_now(
    chrono::Duration::days(chrono::Local::now().weekday().num_days_from_monday().into())
      .checked_add(&get_duration_from_midnight())
      .unwrap(),
  );
}

pub fn get_midnight_datetime() -> chrono::DateTime<chrono::Local> {
  return get_checked_sub_signed_from_now(get_duration_from_midnight());
}

fn get_duration_from_midnight() -> chrono::Duration {
  return chrono::Duration::seconds(
    chrono::Local::now()
      .time()
      .num_seconds_from_midnight()
      .into(),
  );
}

fn get_checked_sub_signed_from_now(duration: chrono::Duration) -> chrono::DateTime<chrono::Local> {
  return chrono::Local::now().checked_sub_signed(duration).unwrap();
}
