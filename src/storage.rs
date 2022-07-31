use std::io::{BufRead, BufReader, Read, Seek, Write};

use crate::{project::Project, state::State, task::Task};

pub struct Storage {
  tasks_file_: std::fs::File,
  tasks_filepath_: String,
  projects_file_: std::fs::File,
  state_file_: std::fs::File,
  state_: State,
  tasks_: Vec<Task>,
}

impl Storage {
  fn restore_state(state_file: &mut std::fs::File) -> State {
    let mut state_str = String::new();
    let _ = state_file.read_to_string(&mut state_str).unwrap();
    let state: State = serde_json::from_str(&state_str).unwrap_or_default();
    return state;
  }

  fn open_file(database_dir: &str, filename: &str) -> (std::path::PathBuf, std::fs::File) {
    let tasks_filepath = std::path::Path::new(database_dir).join(filename);
    return (
      tasks_filepath.clone(),
      std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .read(true)
        .open(tasks_filepath)
        .unwrap(),
    );
  }

  pub fn new(database_dir_path: &str) -> Self {
    let mut state_file = Storage::open_file(database_dir_path, "state.json").1;
    let (tasks_filepath, tasks_file) = Storage::open_file(database_dir_path, "tasks.json");
    let state = Self::restore_state(&mut state_file);
    let mut storage = Self {
      tasks_file_: tasks_file,
      tasks_filepath_: tasks_filepath.to_str().unwrap().to_string(),
      projects_file_: Storage::open_file(database_dir_path, "projects.json").1,
      state_file_: state_file,
      state_: state,
      tasks_: Vec::new(),
    };

    storage.tasks_ = storage.read_tasks();
    return storage;
  }

  pub fn tasks_filepath(&self) -> &str {
    self.tasks_filepath_.as_str()
  }

  pub fn state(&self) -> State {
    self.state_.clone()
  }

  pub fn add_task(&mut self, task: &Task) {
    let task_str = serde_json::to_string(task).unwrap();
    self
      .tasks_file_
      .write_all((task_str + "\n").as_bytes())
      .unwrap();

    self.state_.last_task_id = task.id();
    self.save_state();
  }

  pub fn remove_task(&mut self, task_id: u128) {
    let mut tasks = self.tasks();
    let pos = tasks.iter().position(|t| t.id() == task_id).unwrap();
    tasks.remove(pos);
    self.rewrite_tasks(tasks);
  }

  pub fn replace_task(&mut self, task: Task) -> Result<(), String> {
    let mut tasks = self.tasks();
    if let Some(pos) = tasks.iter().position(|t| t.id() == task.id()) {
      tasks[pos] = task;
      self.rewrite_tasks(tasks);
      return Ok(());
    }
    return Err("source task not found".to_string());
  }

  pub fn tasks(&self) -> Vec<Task> {
    return self.tasks_.clone();
  }

  fn read_tasks(&mut self) -> Vec<Task> {
    self.tasks_file_.rewind().unwrap();
    let mut tasks = Vec::new();
    for line in BufReader::new(&self.tasks_file_).lines() {
      let task: Task = serde_json::from_str(line.unwrap().as_str()).unwrap();
      tasks.push(task);
    }
    return tasks;
  }

  pub fn add_project(&mut self, project: &Project) {
    let project_str = serde_json::to_string(project).unwrap();
    self
      .projects_file_
      .write_all((project_str + "\n").as_bytes())
      .unwrap();

    self.state_.last_project_id = project.id();
    self.save_state();
  }

  pub fn projects(&mut self) -> Vec<Project> {
    self.projects_file_.rewind().unwrap();
    let mut projects = Vec::new();
    for line in BufReader::new(&self.projects_file_).lines() {
      let project: Project = serde_json::from_str(line.unwrap().as_str()).unwrap();
      projects.push(project);
    }
    return projects;
  }

  fn rewrite_tasks(&mut self, tasks: Vec<Task>) {
    let mut new_tasks = tasks;

    new_tasks.sort_by_key(|t| t.id());
    self.tasks_file_.set_len(0).unwrap();

    for task in new_tasks.iter() {
      self.add_task(task);
    }
  }

  fn save_state(&mut self) {
    self.state_file_.set_len(0).unwrap();
    let state_str = serde_json::to_string(&self.state_).unwrap();
    self.state_file_.write_all(state_str.as_bytes()).unwrap();

    self.tasks_file_.flush().unwrap();
    self.state_file_.flush().unwrap();
  }
}
