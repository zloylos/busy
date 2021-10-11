use crate::{category::Category, storage::Storage, task::Task};

pub struct Pomidorka {
  storage_: Storage,
  categories_: Vec<Category>,
  active_tasks_: Vec<Task>,
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
    let categories = storage.categories();
    let active_tasks = tasks
      .iter()
      .filter(|t| !t.time_left().is_zero())
      .map(|t| t.clone())
      .collect();

    Self {
      storage_: storage,
      categories_: categories,
      tasks_: tasks,
      active_tasks_: active_tasks,
    }
  }

  pub fn add_task(&mut self, category_id: u128, description: &str) {
    const DEFAULT_DURATION: f32 = 25.0 * 60.0;
    let task = Task::new(
      self.storage_.state().last_task_id + 1,
      category_id,
      description,
      std::time::Duration::from_secs_f32(DEFAULT_DURATION),
    );
    self.storage_.add_task(&task);
    self.active_tasks_.push(task.clone());
    self.tasks_.push(task);
  }

  pub fn remove_task(&mut self, task_id: u128) -> Result<u128, &str> {
    let active_task_position = self.active_tasks_.iter().position(|t| t.id() == task_id);
    if active_task_position.is_some() {
      self.active_tasks_.remove(active_task_position.unwrap());
    }

    let task_position = self.tasks_.iter().position(|t| t.id() == task_id);
    if task_position.is_none() {
      return Err("task not found");
    }

    self.tasks_.remove(task_position.unwrap());
    self.storage_.remove_task(task_id);

    return Ok(task_id);
  }

  pub fn add_category(&mut self, category_name: &str) {
    let category = Category::new(self.storage_.state().last_category_id + 1, category_name);
    self.storage_.add_category(&category);
    self.categories_.push(category);
  }

  pub fn active_tasks(&self) -> Vec<Task> {
    self.active_tasks_.clone()
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

  pub fn categories(&self) -> Vec<Category> {
    self.categories_.clone()
  }

  pub fn category_by_id(&self, category_id: u128) -> Option<Category> {
    self.categories_.iter().find_map(|c| {
      if c.id() == category_id {
        return Some(c.clone());
      }
      return None;
    })
  }
}
