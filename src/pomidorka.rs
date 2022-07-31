use log::debug;

use crate::{
  project::Project, storage::Storage, sync::GitSyncer, tag::Tag, task::Task, traits::Indexable,
};

const ENV_POMIDORKA_DIR: &str = "POMIDORKA_DIR";
const ENV_POMIDORKA_REMOTE: &str = "POMIDORKA_REMOTE";
const ENV_POMIDORKA_REMOTE_BRANCH: &str = "POMIDORKA_REMOTE_BRANCH";

fn get_env_var(key: &str) -> Option<String> {
  match std::env::var(key) {
    Ok(val) => Some(val),
    Err(_) => None,
  }
}

pub struct Config {
  storage_dir_path: String,
  git_remote: Option<String>,
  git_remote_branch: Option<String>,
}

impl Config {
  pub fn init() -> Self {
    let storage_dir = match get_env_var(ENV_POMIDORKA_DIR) {
      Some(dir) => std::path::Path::new(&dir).to_path_buf(),
      None => std::path::Path::new(std::env::var("HOME").unwrap().as_str()).join(".pomidorka"),
    };

    debug!("storage path is: {:?}", storage_dir);
    std::fs::create_dir_all(&storage_dir).unwrap();

    let storage_path = storage_dir
      .canonicalize()
      .unwrap()
      .to_str()
      .unwrap()
      .to_owned();

    Self {
      storage_dir_path: storage_path,
      git_remote: get_env_var(ENV_POMIDORKA_REMOTE),
      git_remote_branch: get_env_var(ENV_POMIDORKA_REMOTE_BRANCH),
    }
  }
}

pub struct Pomidorka {
  storage_: Storage,
  syncer_: GitSyncer,
  config_: Config,
}

impl Pomidorka {
  pub fn new() -> Self {
    let config = Config::init();
    let syncer = GitSyncer::new(
      &config.storage_dir_path,
      config.git_remote.clone(),
      config.git_remote_branch.clone(),
    );

    Self {
      storage_: Storage::new(&config.storage_dir_path),
      syncer_: syncer,
      config_: config,
    }
  }

  pub fn sync(&mut self) {
    self.syncer_.sync();
    self.storage_ = Storage::new(&self.config_.storage_dir_path);
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
          let new_tag = Tag::new(last_tag_id, tag);
          self.storage_.add_tag(&new_tag);
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

    self.syncer_.commit(&format_task_commit("started", &task));

    return Ok(task);
  }

  pub fn stop(&mut self) -> Result<Task, String> {
    let maybe_active_task = self.active_task();
    if maybe_active_task.is_none() {
      return Err("there is no active task to stop".to_owned());
    }

    let mut active_task = maybe_active_task.unwrap();
    active_task.stop();

    match self.storage_.replace_task(active_task.clone()) {
      Ok(_) => {
        self
          .syncer_
          .commit(&format_task_commit("stopped", &active_task));

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

    match self.storage_.replace_task(active_task.clone()) {
      Ok(_) => {
        self
          .syncer_
          .commit(&format_task_commit("paused", &active_task));

        Ok(active_task)
      }
      Err(err) => Err(err),
    }
  }

  pub fn unpause(&mut self) -> Result<Task, String> {
    const ERR_MSG: &str = "there is no paused task to continue";

    let maybe_active_task = self.active_task();
    if maybe_active_task.is_none() {
      return Err(ERR_MSG.to_owned());
    }

    let mut active_task = maybe_active_task.unwrap();
    if active_task.stop_time().is_none() {
      return Err(ERR_MSG.to_owned());
    }
    active_task.unpause();
    match self.storage_.replace_task(active_task.clone()) {
      Ok(_) => {
        self
          .syncer_
          .commit(&format_task_commit("unpaused", &active_task));

        Ok(active_task)
      }
      Err(err) => Err(err),
    }
  }

  pub fn replace_task(&mut self, task: Task) -> Result<(), String> {
    self.storage_.replace_task(task)
  }

  pub fn replace_project(&mut self, project: Project) -> Result<(), String> {
    self.storage_.replace_project(project)
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

  pub fn find_tags(&self, tag_ids: &Vec<u128>) -> Vec<Tag> {
    self.storage_.find_tags(tag_ids)
  }

  pub fn task_by_id(&self, task_id: u128) -> Option<Task> {
    return self
      .storage_
      .tasks()
      .iter()
      .find(|t| t.id() == task_id)
      .map(|t| t.clone());
  }

  pub fn active_task(&self) -> Option<Task> {
    let tasks = self.storage_.tasks();
    let found_task = tasks
      .iter()
      .find(|t| t.stop_time().is_none() || t.is_paused());
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

  pub fn tag_by_id(&self, tag_id: u128) -> Option<Tag> {
    self.storage_.tags().iter().find_map(|c| {
      if c.id() == tag_id {
        return Some(c.clone());
      }
      return None;
    })
  }

  pub fn find_tag_by_names(&self, tags: &Vec<String>) -> Vec<Tag> {
    self.storage_.find_tag_by_names(tags)
  }

  pub fn replace_tag(&mut self, tag: Tag) -> Result<(), String> {
    self.storage_.replace_tag(tag)
  }

  pub fn tasks_db_filepath(&self) -> &str {
    self.storage_.tasks_filepath()
  }

  pub fn tags_db_filepath(&self) -> &str {
    self.storage_.tags_filepath()
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

fn format_task_commit(prefix: &str, task: &Task) -> String {
  format!(
    "{} task title: {} id: {} project: {}",
    prefix,
    task.title(),
    task.id(),
    task.project_id()
  )
}
