use crate::{tag::Tag, traits::Indexable};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DateTimeInterval {
  pub start_time: chrono::DateTime<chrono::Local>,
  pub stop_time: Option<chrono::DateTime<chrono::Local>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
  id: uuid::Uuid,
  project_id: uuid::Uuid,
  times: Vec<DateTimeInterval>,
  title: String,
  tags: Vec<uuid::Uuid>,
  is_paused: bool,
}

impl Indexable for Task {
  fn id(&self) -> uuid::Uuid {
    self.id
  }
}

impl Task {
  pub fn new(project_id: uuid::Uuid, title: &str, tags: Vec<uuid::Uuid>) -> Self {
    Self {
      id: uuid::Uuid::new_v4(),
      project_id,
      times: vec![DateTimeInterval {
        start_time: chrono::Local::now(),
        stop_time: None,
      }],
      title: title.to_owned(),
      tags,
      is_paused: false,
    }
  }

  pub fn project_id(&self) -> uuid::Uuid {
    self.project_id
  }

  pub fn title(&self) -> &str {
    self.title.as_str()
  }

  pub fn tags(&self) -> &Vec<uuid::Uuid> {
    &self.tags
  }

  pub fn times(&self) -> &Vec<DateTimeInterval> {
    &self.times
  }

  pub fn start_time(&self) -> chrono::DateTime<chrono::Local> {
    self.times.first().unwrap().start_time
  }

  pub fn stop_time(&self) -> Option<chrono::DateTime<chrono::Local>> {
    self.times.last().unwrap().stop_time
  }

  pub fn duration(&self) -> chrono::Duration {
    let mut total_duration = chrono::Duration::zero();
    for interval in self.times.iter() {
      let stop_time = interval.stop_time.unwrap_or(chrono::Local::now());
      total_duration = total_duration
        .checked_add(&stop_time.signed_duration_since(interval.start_time))
        .unwrap();
    }
    return total_duration;
  }

  pub fn stop(&mut self) {
    self.times.last_mut().unwrap().stop_time = Some(chrono::Local::now());
    self.is_paused = false;
  }

  pub fn is_paused(&self) -> bool {
    self.is_paused
  }

  pub fn pause(&mut self) {
    self.stop();
    self.is_paused = true;
  }

  pub fn resume(&mut self) {
    self.times.push(DateTimeInterval {
      start_time: chrono::Local::now(),
      stop_time: None,
    });
    self.is_paused = false;
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskView {
  id: uuid::Uuid,
  project_id: uuid::Uuid,
  times: Vec<DateTimeInterval>,
  title: String,
  tags: Vec<String>,
  is_paused: bool,
}

impl TaskView {
  pub fn from_task(task: &Task, all_tags: &Vec<Tag>) -> Self {
    TaskView {
      id: task.id().clone(),
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

  pub fn resolve_new_tags(&self, existing_tags: &Vec<Tag>) -> Vec<String> {
    let mut new_tags = vec![];
    for tag_name in self.tags.iter() {
      if existing_tags
        .iter()
        .find(|tag| tag.name() == tag_name)
        .is_none()
      {
        new_tags.push(tag_name.to_owned());
      }
    }
    return new_tags;
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
      id: self.id,
      project_id: self.project_id,
      times: self.times.clone(),
      title: self.title.clone(),
      tags: tag_ids,
      is_paused: self.is_paused,
    }
  }
}
