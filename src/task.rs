use crate::traits::Indexable;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
  id_: u128,
  project_id_: u128,
  start_time_: chrono::DateTime<chrono::Local>,
  stop_time_: Option<chrono::DateTime<chrono::Local>>,
  title_: String,
  tags_: Vec<String>,
}

impl Indexable for Task {
  fn id(&self) -> u128 {
    self.id_
  }
}

impl Task {
  pub fn new(id: u128, project_id: u128, title: &str, tags: Vec<String>) -> Self {
    Self {
      id_: id,
      project_id_: project_id,
      start_time_: chrono::Local::now(),
      stop_time_: None,
      title_: title.to_owned(),
      tags_: tags,
    }
  }

  pub fn project_id(&self) -> u128 {
    self.project_id_
  }

  pub fn title(&self) -> &str {
    self.title_.as_str()
  }

  pub fn tags(&self) -> &Vec<String> {
    &self.tags_
  }

  pub fn start_time(&self) -> chrono::DateTime<chrono::Local> {
    self.start_time_
  }

  pub fn stop_time(&self) -> Option<chrono::DateTime<chrono::Local>> {
    self.stop_time_
  }

  pub fn duration(&self) -> chrono::Duration {
    let stop_time = self.stop_time_.unwrap_or(chrono::Local::now());
    return stop_time.signed_duration_since(self.start_time_);
  }

  pub fn stop(&mut self) {
    self.stop_time_ = Some(chrono::Local::now());
  }
}
