use crate::{project::Project, storage::Storage, tag::Tag, task::Task, traits::Indexable};

pub struct Pomidorka {
  storage_: Storage,
  projects_: Vec<Project>,
  tasks_: Vec<Task>,
}

impl Pomidorka {
  pub fn new() -> Self {
    let home_dir = std::env!("HOME");
    let default_database_path = std::path::Path::new(home_dir).join(".pomidorka");
    let _ = std::fs::create_dir_all(&default_database_path);
    let storage = Storage::new(
      default_database_path
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap(),
    );

    let tasks = storage.tasks();
    let projects = storage.projects();

    Self {
      storage_: storage,
      projects_: projects,
      tasks_: tasks,
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
    self.tasks_.push(task.clone());
    return Ok(task);
  }

  pub fn stop(&mut self) -> Result<Task, String> {
    let active_task_opt = self.tasks_.iter_mut().find(|t| t.stop_time().is_none());
    if active_task_opt.is_none() {
      return Err("active task not found, start it firstly".to_string());
    }

    let active_task = active_task_opt.unwrap();
    active_task.stop();

    self.storage_.remove_task(active_task.id());
    self.storage_.add_task(active_task);

    return Ok(active_task.clone());
  }

  pub fn replace_task(&mut self, task: Task) -> Result<(), String> {
    self.storage_.replace_task(task);
    self.tasks_ = self.storage_.tasks();
    return Ok(());
  }

  pub fn remove_task(&mut self, task_id: u128) -> Result<u128, &str> {
    let task_position = self.tasks_.iter().position(|t| t.id() == task_id);
    if task_position.is_none() {
      return Err("task not found");
    }

    self.tasks_.remove(task_position.unwrap());
    self.storage_.remove_task(task_id);

    return Ok(task_id);
  }

  pub fn tasks(&self, period: chrono::Duration) -> Vec<Task> {
    let current_time = chrono::Local::now();
    self
      .tasks_
      .iter()
      .filter(|t| current_time.signed_duration_since(t.start_time()) < period)
      .map(|t| t.clone())
      .collect()
  }

  pub fn task_by_id(&self, task_id: u128) -> Option<Task> {
    return self
      .tasks_
      .iter()
      .find(|t| t.id() == task_id)
      .map(|t| t.clone());
  }

  pub fn active_task(&mut self) -> Option<&Task> {
    self.tasks_.iter().find(|t| t.stop_time().is_none())
  }

  pub fn projects(&self) -> Vec<Project> {
    self.projects_.clone()
  }

  pub fn tags(&self) -> Vec<Tag> {
    self.storage_.tags().clone()
  }

  pub fn tasks_db_filepath(&self) -> &str {
    self.storage_.tasks_filepath()
  }

  fn add_project(&mut self, project_name: &str) -> Project {
    let project = Project::new(self.storage_.state().last_project_id + 1, project_name);
    self.storage_.add_project(&project);
    self.projects_.push(project.clone());

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
    return self.projects_.iter().find_map(|p| {
      if p.name() == project_name {
        return Some(p.clone());
      }
      return None;
    });
  }

  pub fn project_by_id(&self, project_id: u128) -> Option<Project> {
    self.projects_.iter().find_map(|c| {
      if c.id() == project_id {
        return Some(c.clone());
      }
      return None;
    })
  }
}
