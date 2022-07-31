use crate::{project::Project, storage::Storage, task::Task};

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
    let mut storage = Storage::new(
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

  pub fn start(
    &mut self,
    project_name: &str,
    title: &str,
    tags: Vec<String>,
  ) -> Result<Task, &str> {
    if !self.active_task().is_none() {
      return Err("active task already exists, stop it firstly");
    }
    let project = self.upsert_project(project_name);
    let task = Task::new(
      self.storage_.state().last_task_id + 1,
      project.id(),
      title,
      tags,
    );
    self.storage_.add_task(&task);
    self.tasks_.push(task.clone());
    return Ok(task);
  }

  pub fn stop(&mut self) -> Result<Task, &str> {
    let active_task_opt = self.tasks_.iter_mut().find(|t| t.stop_time().is_none());
    if active_task_opt.is_none() {
      return Err("active task not found, start it firstly");
    }

    let active_task = active_task_opt.unwrap();
    active_task.stop();

    self.storage_.remove_task(active_task.id());
    self.storage_.add_task(active_task);

    return Ok(active_task.clone());
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

  pub fn active_task(&mut self) -> Option<&Task> {
    self.tasks_.iter().find(|t| t.stop_time().is_none())
  }

  pub fn projects(&self) -> Vec<Project> {
    self.projects_.clone()
  }

  fn add_project(&mut self, project_name: &str) -> Project {
    let project = Project::new(self.storage_.state().last_project_id + 1, project_name);
    self.storage_.add_project(&project);
    self.projects_.push(project.clone());

    return project;
  }

  fn upsert_project(&mut self, name: &str) -> Project {
    let project = self.projects_.iter().find_map(|p| {
      if p.name() == name {
        return Some(p.clone());
      }
      return None;
    });
    if project.is_none() {
      return self.add_project(name);
    }
    return project.unwrap();
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
