use crate::{tag::Tag, traits::Indexable};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
  id_: u128,
  project_id_: u128,
  start_time_: chrono::DateTime<chrono::Local>,
  stop_time_: Option<chrono::DateTime<chrono::Local>>,
  title_: String,
  tags_: Vec<u128>,
}

impl Indexable for Task {
  fn id(&self) -> u128 {
    self.id_
  }
}

impl Task {
  pub fn new(id: u128, project_id: u128, title: &str, tags: Vec<u128>) -> Self {
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

  pub fn tags(&self) -> &Vec<u128> {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskView {
  id: u128,
  project_id: u128,
  start_time: chrono::DateTime<chrono::Local>,
  stop_time: Option<chrono::DateTime<chrono::Local>>,
  title: String,
  tags: Vec<String>,
}

impl TaskView {
  pub fn from_task(task: &Task, all_tags: &Vec<Tag>) -> Self {
    TaskView {
      id: task.id(),
      project_id: task.project_id(),
      start_time: task.start_time(),
      stop_time: task.stop_time(),
      title: task.title().to_owned(),
      tags: all_tags
        .iter()
        .filter(|tag| task.tags().contains(&tag.id()))
        .map(|tag| tag.name().to_owned())
        .collect(),
    }
  }

  pub fn to_task(&self, all_tags: &Vec<Tag>) -> Task {
    // TODO: upsert new tags after edit
    let tag_ids = self
      .tags
      .iter()
      .map(|tag_name| {
        let found_tag = all_tags
          .iter()
          .find(|t| t.name() == tag_name)
          .expect("add new tags unsupported yet");
        return found_tag.id();
      })
      .collect();

    Task {
      id_: self.id,
      project_id_: self.project_id,
      start_time_: self.start_time,
      stop_time_: self.stop_time,
      title_: self.title.clone(),
      tags_: tag_ids,
    }
  }
}
