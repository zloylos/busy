use std::io::{Seek, Write};

use log::debug;

use crate::{project::Project, tag::Tag, task::Task, traits::Indexable};

pub struct Storage {
  tasks: StorageItem<Task>,
  projects: StorageItem<Project>,
  tags: StorageItem<Tag>,
}

impl Storage {
  pub fn new(database_folder: &str) -> Self {
    let database_path = std::path::Path::new(database_folder);

    let join_path = |filename: &str| database_path.join(filename).to_str().unwrap().to_owned();

    Self {
      tasks: StorageItem::new(join_path("tasks.json").as_str()),
      projects: StorageItem::new(join_path("projects.json").as_str()),
      tags: StorageItem::new(join_path("tags.json").as_str()),
    }
  }

  pub fn ids(&self) -> Vec<uuid::Uuid> {
    // TODO: optimize
    let mut ids = Vec::new();
    for task in self.tasks() {
      ids.push(task.id().clone());
    }
    for project in self.projects() {
      ids.push(project.id().clone());
    }
    for tag in self.tags() {
      ids.push(tag.id().clone());
    }
    return ids;
  }

  pub fn tasks_filepath(&self) -> &str {
    self.tasks.storage_path()
  }

  pub fn tags_filepath(&self) -> &str {
    self.tags.storage_path()
  }

  pub fn add_task(&mut self, task: &Task) {
    self.tasks.add(task.clone());
  }

  pub fn remove_task(&mut self, task_id: uuid::Uuid) -> Result<(), String> {
    self.tasks.remove(task_id)
  }

  pub fn replace_task(&mut self, task: &Task) -> Result<(), String> {
    self.tasks.replace(task)
  }

  pub fn tasks(&self) -> Vec<Task> {
    let mut tasks = self.tasks.all();
    tasks.sort_by(|a, b| a.start_time().cmp(&b.start_time()));
    return tasks;
  }

  pub fn find_tag_by_name(&self, tag: &str) -> Option<Tag> {
    match self.tags.all().iter().find(|t| t.name() == tag) {
      Some(found_tag) => Some(found_tag.clone()),
      _ => None,
    }
  }

  pub fn find_tag_by_names(&self, tag_strs: &Vec<String>) -> Vec<Tag> {
    let mut tags = Vec::with_capacity(tag_strs.len());
    for tag_str in tag_strs.iter() {
      let found_tag = self.find_tag_by_name(tag_str);
      if found_tag.is_some() {
        tags.push(found_tag.unwrap().clone());
      }
    }
    return tags;
  }

  pub fn find_tags(&self, tag_ids: &Vec<uuid::Uuid>) -> Vec<Tag> {
    let mut tags = Vec::new();
    for tag_id in tag_ids.iter() {
      match self.tag_by_id(*tag_id) {
        Some(found_tag) => tags.push(found_tag.clone()),
        _ => {}
      };
    }
    return tags;
  }

  pub fn tag_by_id(&self, id: uuid::Uuid) -> Option<&Tag> {
    self.tags.get_by_id(id)
  }

  pub fn add_tag(&mut self, tag: &Tag) {
    self.tags.add(tag.clone());
  }

  pub fn tags(&self) -> Vec<Tag> {
    self.tags.all()
  }

  pub fn replace_tag(&mut self, tag: &Tag) -> Result<(), String> {
    self.tags.replace(tag)
  }

  pub fn add_project(&mut self, project: &Project) {
    self.projects.add(project.clone());
  }

  pub fn projects(&self) -> Vec<Project> {
    self.projects.all()
  }

  pub fn replace_project(&mut self, project: &Project) -> Result<(), String> {
    self.projects.replace(project)
  }
}

struct StorageItem<T> {
  filepath: String,
  file: std::fs::File,
  buffer: Vec<T>,
}

impl<T> StorageItem<T>
where
  T: Indexable + Clone + serde::de::DeserializeOwned + serde::ser::Serialize,
{
  fn new(filepath: &str) -> Self {
    let mut storage_item = Self {
      filepath: filepath.to_owned(),
      file: std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(filepath)
        .unwrap(),
      buffer: Vec::new(),
    };

    storage_item.restore();

    debug!(
      "restored {} items from: {}",
      storage_item.buffer.len(),
      filepath
    );

    return storage_item;
  }

  fn storage_path(&self) -> &str {
    self.filepath.as_str()
  }

  fn get_by_id(&self, id: uuid::Uuid) -> Option<&T> {
    self.buffer.iter().find(|item| item.id() == id)
  }

  fn restore(&mut self) {
    self.file.rewind().unwrap();
    self.buffer = serde_json::from_reader(&self.file).unwrap_or_default();
  }

  fn add(&mut self, item: T) {
    self.buffer.push(item.clone());
    self.flush();
  }

  fn remove(&mut self, id: uuid::Uuid) -> Result<(), String> {
    let position = self.position_by_id(id);
    if position.is_none() {
      return Err(format!("task with id: {} not found", id));
    }

    self.buffer.remove(position.unwrap());
    self.flush();

    Ok(())
  }

  fn replace(&mut self, item: &T) -> Result<(), String> {
    let position = self.position_by_id(item.id().clone());
    if position.is_none() {
      return Err(format!("task with id: {} not found", item.id()));
    }

    self.buffer[position.unwrap()] = item.clone();
    self.flush();

    Ok(())
  }

  fn all(&self) -> Vec<T> {
    self.buffer.clone()
  }

  fn position_by_id(&self, id: uuid::Uuid) -> Option<usize> {
    self.buffer.iter().position(|item| item.id() == id)
  }

  fn flush(&mut self) {
    self.file.set_len(0).unwrap();
    self.file.rewind().unwrap();
    self
      .file
      .write_all(
        serde_json::to_string_pretty(&self.buffer)
          .unwrap()
          .as_bytes(),
      )
      .expect("can't write information to db");

    self.file.flush().expect("save db erorr");
  }
}

#[cfg(test)]
mod test {
  use super::{Indexable, StorageItem};

  #[derive(Clone, serde::Serialize, serde::Deserialize)]
  struct TestType {
    id: uuid::Uuid,
    title: String,
  }

  impl TestType {
    fn new(title: &str) -> Self {
      Self {
        id: uuid::Uuid::new_v4(),
        title: title.to_string(),
      }
    }

    fn title(&self) -> &str {
      self.title.as_str()
    }
  }

  impl Indexable for TestType {
    fn id(&self) -> uuid::Uuid {
      self.id
    }
  }

  fn get_new_storage() -> StorageItem<TestType> {
    let tmp_file = tempfile::Builder::new()
      .prefix("busy")
      .suffix(".json")
      .tempfile()
      .unwrap();

    StorageItem::<TestType>::new(tmp_file.into_temp_path().to_str().unwrap())
  }

  #[test]
  fn storage_item_add() {
    let mut storage = get_new_storage();
    let new_item = TestType::new("Hello");

    storage.add(new_item);
    let all_items = storage.all();

    assert_eq!(all_items.len(), 1);
  }

  #[test]
  fn storage_item_remove() {
    let mut storage = get_new_storage();
    let new_item = TestType::new("Hello");
    let id = new_item.id().clone();

    storage.add(new_item);
    storage.remove(id.clone()).unwrap();
    let all_items = storage.all();

    assert_eq!(all_items.is_empty(), true);
  }

  #[test]
  fn storage_item_remove_from_empty_storage() {
    let mut storage = get_new_storage();
    storage
      .remove(uuid::Uuid::new_v4())
      .expect_err("shouldn't remove from empty storage");
  }

  #[test]
  fn storage_item_replace() {
    let mut storage = get_new_storage();
    let item = TestType::new("Hello");
    let id = item.id();
    storage.add(item);

    let mut new_item = TestType::new("Hello, world!");
    new_item.id = id;

    storage.replace(&new_item).unwrap();

    let all_items = storage.all();
    assert_eq!(all_items.len(), 1);
    assert_eq!(all_items[0].title(), "Hello, world!");
  }
}
