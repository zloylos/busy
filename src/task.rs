use chrono;
use serde;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
  id_: u128,
  category_id_: u128,
  start_time_: chrono::DateTime<chrono::Local>,
  duration_: std::time::Duration,
  description_: String,
}

impl Task {
  pub fn new(
    id: u128,
    category_id: u128,
    description: &str,
    duration: std::time::Duration,
  ) -> Self {
    Self {
      id_: id,
      start_time_: chrono::Local::now(),
      duration_: duration,
      category_id_: category_id,
      description_: description.to_owned(),
    }
  }

  pub fn id(&self) -> u128 {
    self.id_
  }

  pub fn category_id(&self) -> u128 {
    self.category_id_
  }

  pub fn description(&self) -> &str {
    self.description_.as_str()
  }

  pub fn start_time(&self) -> chrono::DateTime<chrono::Local> {
    self.start_time_
  }

  pub fn duration(&self) -> std::time::Duration {
    self.duration_
  }

  pub fn time_left(&self) -> std::time::Duration {
    self
      .duration_
      .checked_sub(
        chrono::Local::now()
          .signed_duration_since(self.start_time_)
          .to_std()
          .unwrap(),
      )
      .unwrap_or_default()
  }

  pub fn stop(&mut self) {
    self.duration_ = chrono::Local::now()
      .signed_duration_since(self.start_time_)
      .to_std()
      .unwrap();
  }
}
