use crate::{project::Project, storage::Storage, tag::Tag, task::Task, traits::Indexable};

fn get_storage_dir_path() -> String {
  let storage_dir = match std::env::var("POMIDORKA_DIR") {
    Ok(dir) => std::path::Path::new(&dir).to_path_buf(),
    Err(_) => std::path::Path::new(std::env::var("HOME").unwrap().as_str()).join(".pomidorka"),
  };

  std::fs::create_dir_all(&storage_dir).unwrap();

  return storage_dir
    .canonicalize()
    .unwrap()
    .to_str()
    .unwrap()
    .to_owned();
}

pub struct Pomidorka {
  storage_: Storage,
}

impl Pomidorka {
  pub fn new() -> Self {
    Self {
      storage_: Storage::new(&get_storage_dir_path()),
    }
  }

  pub fn storage(&self) -> &Storage {
    &self.storage_
  }

  fn upsert_tags(&mut self, tags: Vec<String>) -> Vec<u128> {
    let state = self.storage_.state();
    let mut pushed_ids = Vec::new();
    let mut last_tag_id = state.last_tag_id + 1;
    for tag in tags.iter() {
      match self.storage_.find_tag_by_name(tag) {
        Some(found_tag) => {
          pushed_ids.push(found_tag.id());
        }
        None => {
          self.storage_.add_tag(&Tag::new(last_tag_id, tag));
          pushed_ids.push(last_tag_id);
          last_tag_id += 1;
        }
      }
    }
    return pushed_ids;
  }

  pub fn start(
    &mut self,
    project_name: &str,
    title: &str,
    tags: Vec<String>,
  ) -> Result<Task, String> {
    if !self.active_task().is_none() {
      return Err("active task already exists, stop it firstly".to_string());
    }
    let project = self.upsert_project(project_name);
    let task = Task::new(
      self.storage_.state().last_task_id + 1,
      project.id(),
      title,
      self.upsert_tags(tags),
    );
    self.storage_.add_task(&task);
    return Ok(task);
  }

  pub fn stop(&mut self) -> Result<Task, String> {
    let mut tasks = self.storage_.tasks();
    let active_task_opt = tasks.iter_mut().find(|t| t.stop_time().is_none());

    if active_task_opt.is_none() {
      return Err("active task not found, start it firstly".to_string());
    }

    let active_task = active_task_opt.unwrap();
    active_task.stop();

    match self.storage_.replace_task(active_task.clone()) {
      Ok(_) => Ok(active_task.clone()),
      Err(err) => Err(err),
    }
  }

  pub fn replace_task(&mut self, task: Task) -> Result<(), String> {
    self.storage_.replace_task(task)
  }

  pub fn remove_task(&mut self, task_id: u128) -> Result<(), String> {
    self.storage_.remove_task(task_id)
  }

  pub fn tasks(&self, period: chrono::Duration) -> Vec<Task> {
    let current_time = chrono::Local::now();
    self
      .storage_
      .tasks()
      .iter()
      .filter(|t| current_time.signed_duration_since(t.start_time()) < period)
      .map(|t| t.clone())
      .collect()
  }

  pub fn task_by_id(&self, task_id: u128) -> Option<Task> {
    return self
      .storage_
      .tasks()
      .iter()
      .find(|t| t.id() == task_id)
      .map(|t| t.clone());
  }

  pub fn active_task(&mut self) -> Option<Task> {
    let tasks = self.storage_.tasks();
    let found_task = tasks.iter().find(|t| t.stop_time().is_none());
    match found_task {
      Some(task) => Some(task.clone()),
      None => None,
    }
  }

  pub fn projects(&self) -> Vec<Project> {
    self.storage_.projects()
  }

  pub fn tags(&self) -> Vec<Tag> {
    self.storage_.tags()
  }

  pub fn find_tag_by_names(&self, tags: &Vec<String>) -> Vec<Tag> {
    self.storage_.find_tag_by_names(tags)
  }

  pub fn tasks_db_filepath(&self) -> &str {
    self.storage_.tasks_filepath()
  }

  fn add_project(&mut self, project_name: &str) -> Project {
    let project = Project::new(self.storage_.state().last_project_id + 1, project_name);
    self.storage_.add_project(&project);
    return project;
  }

  fn upsert_project(&mut self, project_name: &str) -> Project {
    let project = self.project_by_name(project_name);
    if project.is_none() {
      return self.add_project(project_name);
    }
    return project.unwrap();
  }

  pub fn project_by_name(&self, project_name: &str) -> Option<Project> {
    return self.storage_.projects().iter().find_map(|p| {
      if p.name() == project_name {
        return Some(p.clone());
      }
      return None;
    });
  }

  pub fn project_by_id(&self, project_id: u128) -> Option<Project> {
    self.storage_.projects().iter().find_map(|c| {
      if c.id() == project_id {
        return Some(c.clone());
      }
      return None;
    })
  }
}
