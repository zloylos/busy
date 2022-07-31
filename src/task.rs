use crate::{tag::Tag, traits::Indexable};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DateTimeInterval {
  pub start_time: chrono::DateTime<chrono::Local>,
  pub stop_time: Option<chrono::DateTime<chrono::Local>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
  id_: u128,
  project_id_: u128,
  times_: Vec<DateTimeInterval>,
  title_: String,
  tags_: Vec<u128>,
  is_paused_: bool,
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
      times_: vec![DateTimeInterval {
        start_time: chrono::Local::now(),
        stop_time: None,
      }],
      title_: title.to_owned(),
      tags_: tags,
      is_paused_: false,
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

  pub fn times(&self) -> &Vec<DateTimeInterval> {
    &self.times_
  }

  pub fn start_time(&self) -> chrono::DateTime<chrono::Local> {
    self.times_.first().unwrap().start_time
  }

  pub fn stop_time(&self) -> Option<chrono::DateTime<chrono::Local>> {
    self.times_.last().unwrap().stop_time
  }

  pub fn duration(&self) -> chrono::Duration {
    let mut total_duration = chrono::Duration::zero();
    for interval in self.times_.iter() {
      let stop_time = interval.stop_time.unwrap_or(chrono::Local::now());
      total_duration = total_duration
        .checked_add(&stop_time.signed_duration_since(interval.start_time))
        .unwrap();
    }
    return total_duration;
  }

  pub fn stop(&mut self) {
    self.times_.last_mut().unwrap().stop_time = Some(chrono::Local::now());
  }

  pub fn is_paused(&self) -> bool {
    self.is_paused_
  }

  pub fn pause(&mut self) {
    self.stop();
    self.is_paused_ = true;
  }

  pub fn unpause(&mut self) {
    self.times_.push(DateTimeInterval {
      start_time: chrono::Local::now(),
      stop_time: None,
    });
    self.is_paused_ = false;
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskView {
  id: u128,
  project_id: u128,
  times: Vec<DateTimeInterval>,
  title: String,
  tags: Vec<String>,
  is_paused: bool,
}

impl TaskView {
  pub fn from_task(task: &Task, all_tags: &Vec<Tag>) -> Self {
    TaskView {
      id: task.id(),
      project_id: task.project_id(),
      times: task.times().clone(),
      title: task.title().to_owned(),
      tags: all_tags
        .iter()
        .filter(|tag| task.tags().contains(&tag.id()))
        .map(|tag| tag.name().to_owned())
        .collect(),
      is_paused: task.is_paused(),
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
      times_: self.times.clone(),
      title_: self.title.clone(),
      tags_: tag_ids,
      is_paused_: self.is_paused,
    }
  }
}
