use log::debug;

use crate::{
  duration::Period,
  project::Project,
  storage::{JsonStorage, Storage},
  sync::Syncer,
  sync::{EmptySyncer, GitSyncer, SyncerConfig},
  tag::Tag,
  task::Task,
  traits::Indexable,
  Config,
};

pub struct Busy {
  storage: Box<dyn Storage>,
  syncer: Box<dyn Syncer>,
  config: Config,
}

impl Busy {
  pub fn new() -> Self {
    let config = Config::new();

    debug!("busy data folder: {}", config.storage_dir_path);
    std::fs::create_dir_all(&config.storage_dir_path).unwrap();

    let syncer: Box<dyn Syncer> = match config.syncer.clone() {
      SyncerConfig::Empty => Box::new(EmptySyncer::new()),
      SyncerConfig::Git {
        remote,
        remote_branch,
        key_file,
      } => Box::new(GitSyncer::new(
        &config.storage_dir_path,
        key_file,
        Some(remote),
        remote_branch,
      )),
    };

    Self {
      storage: Box::new(JsonStorage::new(&config.storage_dir_path)),
      syncer,
      config,
    }
  }

  pub fn sync(&mut self) -> std::io::Result<String> {
    self.syncer.sync()?;
    self.storage = Box::new(JsonStorage::new(&self.config.storage_dir_path));

    return Ok("sync success".to_string());
  }

  pub fn push_force(&mut self) -> std::io::Result<String> {
    self.syncer.push_force()
  }

  pub fn pull_force(&mut self) -> std::io::Result<String> {
    self.syncer.pull_force()
  }

  pub fn ids(&self) -> Vec<uuid::Uuid> {
    self.storage.ids()
  }

  pub fn upsert_tags(&mut self, tags: Vec<String>) -> Vec<uuid::Uuid> {
    let mut pushed_ids = Vec::new();
    for tag in tags.iter() {
      match self.storage.find_tag_by_name(tag) {
        Some(found_tag) => {
          pushed_ids.push(found_tag.id().clone());
        }
        None => {
          let new_tag = Tag::new(tag);
          self.storage.add_tag(&new_tag);
          pushed_ids.push(new_tag.id().clone());
        }
      }
    }
    return pushed_ids;
  }

  pub fn add(
    &mut self,
    project_name: &str,
    title: &str,
    tags: Vec<String>,
    start_time: chrono::DateTime<chrono::Local>,
    finish_time: chrono::DateTime<chrono::Local>,
  ) -> Result<Task, String> {
    let project = self.upsert_project(project_name);
    let task = Task::new(
      project.id(),
      title,
      self.upsert_tags(tags),
      Some(start_time),
      Some(finish_time),
    );
    self.storage.add_task(&task);

    self.commit(&format_task_commit("added", &task));
    return Ok(task);
  }

  pub fn start(
    &mut self,
    project_name: &str,
    title: &str,
    tags: Vec<String>,
    start_time: Option<chrono::DateTime<chrono::Local>>,
  ) -> Result<Task, String> {
    if !self.active_task().is_none() {
      return Err("active task already exists, stop it firstly".to_string());
    }
    let project = self.upsert_project(project_name);
    let task = Task::new(
      project.id(),
      title,
      self.upsert_tags(tags),
      start_time,
      None,
    );
    self.storage.add_task(&task);

    self.commit(&format_task_commit("started", &task));

    return Ok(task);
  }

  pub fn stop(&mut self) -> Result<Task, String> {
    let maybe_active_task = self.active_task();
    if maybe_active_task.is_none() {
      return Err("there is no active task to stop".to_owned());
    }

    let mut active_task = maybe_active_task.unwrap();
    active_task.stop();

    match self.storage.replace_task(&active_task.clone()) {
      Ok(_) => {
        self.commit(&format_task_commit("stopped", &active_task));
        Ok(active_task)
      }
      Err(err) => Err(err),
    }
  }

  pub fn pause(&mut self) -> Result<Task, String> {
    let maybe_active_task = self.active_task();
    if maybe_active_task.is_none() {
      return Err("there is no active task to pause".to_owned());
    }

    let mut active_task = maybe_active_task.unwrap();
    active_task.pause();

    match self.storage.replace_task(&active_task) {
      Ok(_) => {
        self.commit(&format_task_commit("paused", &active_task));
        Ok(active_task)
      }
      Err(err) => Err(err),
    }
  }

  pub fn resume(&mut self) -> Result<Task, String> {
    const ERR_MSG: &str = "there is no paused task to continue";

    let maybe_active_task = self.active_task();
    if maybe_active_task.is_none() {
      return Err(ERR_MSG.to_owned());
    }

    let mut active_task = maybe_active_task.unwrap();
    if active_task.stop_time().is_none() {
      return Err(ERR_MSG.to_owned());
    }
    active_task.resume();
    match self.storage.replace_task(&active_task) {
      Ok(_) => {
        self.commit(&format_task_commit("continue", &active_task));
        Ok(active_task)
      }
      Err(err) => Err(err),
    }
  }

  pub fn continue_task(&mut self, task_id: uuid::Uuid) -> Result<Task, String> {
    if self.active_task().is_some() {
      return Err("found active task, please stop it firstly".to_owned());
    }

    let maybe_task_to_continue = self.task_by_id(task_id);
    if maybe_task_to_continue.is_none() {
      return Err(format!("task with id: {} not found", task_id));
    }
    let existing_task = maybe_task_to_continue.unwrap();
    let new_task = Task::new(
      existing_task.project_id(),
      existing_task.title(),
      existing_task.tags().clone(),
      None,
      None,
    );
    self.storage.add_task(&new_task);
    self.commit(&format_task_commit("continue", &new_task));
    return Ok(new_task);
  }

  pub fn replace_task(&mut self, task: &Task) -> Result<(), String> {
    match self.storage.replace_task(task) {
      Ok(_) => {
        self.commit(&format_task_commit("replace", &task));
        return Ok(());
      }
      Err(err) => Err(err),
    }
  }

  pub fn replace_project(&mut self, project: &Project) -> Result<(), String> {
    match self.storage.replace_project(project) {
      Ok(_) => {
        self.commit(&format!(
          "replace task, name: {} id: {}",
          project.name(),
          project.id()
        ));
        return Ok(());
      }
      Err(err) => Err(err),
    }
  }

  pub fn remove_task(&mut self, task_id: uuid::Uuid) -> Result<(), String> {
    self.storage.remove_task(task_id)
  }

  pub fn tasks(&self, period: Period) -> Vec<Task> {
    self
      .storage
      .tasks()
      .iter()
      .filter(|t| {
        let mut within_the_period = period.contains(&t.start_time());
        if t.stop_time().is_some() {
          within_the_period = within_the_period && period.contains(t.stop_time().as_ref().unwrap());
        }
        return within_the_period;
      })
      .map(|t| t.clone())
      .collect()
  }

  pub fn find_tags(&self, tag_ids: &Vec<uuid::Uuid>) -> Vec<Tag> {
    self.storage.find_tags(tag_ids)
  }

  pub fn task_by_id(&self, task_id: uuid::Uuid) -> Option<Task> {
    return self
      .storage
      .tasks()
      .iter()
      .find(|t| t.id() == task_id)
      .map(|t| t.clone());
  }

  pub fn active_task(&self) -> Option<Task> {
    let tasks = self.storage.tasks();
    let found_task = tasks
      .iter()
      .find(|t| t.stop_time().is_none() || t.is_paused());
    match found_task {
      Some(task) => Some(task.clone()),
      None => None,
    }
  }

  pub fn projects(&self) -> Vec<Project> {
    self.storage.projects()
  }

  pub fn tags(&self) -> Vec<Tag> {
    self.storage.tags()
  }

  pub fn tag_by_id(&self, tag_id: uuid::Uuid) -> Option<Tag> {
    self.storage.tags().iter().find_map(|c| {
      if c.id() == tag_id {
        return Some(c.clone());
      }
      return None;
    })
  }

  pub fn find_tag_by_names(&self, tags: &Vec<String>) -> Vec<Tag> {
    self.storage.find_tag_by_names(tags)
  }

  pub fn replace_tag(&mut self, tag: &Tag) -> Result<(), String> {
    self.storage.replace_tag(tag)
  }

  pub fn all_tasks(&self) -> Vec<Task> {
    return self.storage.tasks();
  }

  pub fn all_tags(&self) -> Vec<Tag> {
    return self.storage.tags();
  }

  pub fn replace_tags(&mut self, tags: Vec<Tag>) {
    self.storage.replace_tags(tags);
    self.commit("Edit all tags");
  }

  pub fn replace_tasks(&mut self, tasks: Vec<Task>) {
    self.storage.replace_tasks(tasks);
    self.commit("Edit all tasks");
  }

  fn add_project(&mut self, project_name: &str) -> Project {
    let project = Project::new(project_name);
    self.storage.add_project(&project);
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
    return self.storage.projects().iter().find_map(|p| {
      if p.name() == project_name {
        return Some(p.clone());
      }
      return None;
    });
  }

  pub fn project_by_id(&self, project_id: uuid::Uuid) -> Option<Project> {
    self.storage.projects().iter().find_map(|c| {
      if c.id() == project_id {
        return Some(c.clone());
      }
      return None;
    })
  }

  fn commit(&mut self, msg: &str) {
    match self.syncer.commit(msg) {
      Err(err) => println!("commit err: {} msg: {}", err, msg),
      _ => {}
    };
  }
}

fn format_task_commit(prefix: &str, task: &Task) -> String {
  format!(
    "{} task title: {} id: {} project: {}",
    prefix,
    task.title(),
    task.id(),
    task.project_id()
  )
}
