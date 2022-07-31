use std::io::{Seek, Write};

use crate::{project::Project, state::State, tag::Tag, task::Task, traits::Indexable};

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

  pub fn tasks_filepath(&self) -> &str {
    self.tasks.storage_path()
  }

  pub fn state(&self) -> State {
    State {
      last_task_id: self.tasks.last_id(),
      last_project_id: self.projects.last_id(),
      last_tag_id: self.tags.last_id(),
    }
  }

  pub fn add_task(&mut self, task: &Task) {
    self.tasks.add(task.clone());
  }

  pub fn remove_task(&mut self, task_id: u128) {
    self.tasks.remove(task_id);
  }

  pub fn replace_task(&mut self, task: Task) {
    self.tasks.replace(task);
  }

  pub fn tasks(&self) -> Vec<Task> {
    self.tasks.all()
  }

  pub fn add_tag(&mut self, tag: &Tag) {
    self.tags.add(tag.clone());
  }

  pub fn tags(&self) -> Vec<Tag> {
    self.tags.all()
  }

  pub fn add_project(&mut self, project: &Project) {
    self.projects.add(project.clone());
  }

  pub fn projects(&self) -> Vec<Project> {
    self.projects.all()
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
    return storage_item;
  }

  fn last_id(&self) -> u128 {
    let last_item = self.buffer.last();
    if last_item.is_some() {
      return last_item.unwrap().id();
    }
    return 0;
  }

  fn storage_path(&self) -> &str {
    self.filepath.as_str()
  }

  fn restore(&mut self) {
    self.file.rewind().unwrap();
    self.buffer = serde_json::from_reader(&self.file).unwrap_or_default();
  }

  fn add(&mut self, item: T) {
    self.buffer.push(item.clone());
    self.flush();
  }

  fn remove(&mut self, id: u128) {
    self.buffer.remove(self.position_by_id(id));
    self.flush();
  }

  fn replace(&mut self, item: T) {
    let position = self.position_by_id(item.id());
    self.buffer[position] = item;
    self.flush();
  }

  fn all(&self) -> Vec<T> {
    self.buffer.clone()
  }

  fn position_by_id(&self, id: u128) -> usize {
    self.buffer.iter().position(|item| item.id() == id).unwrap()
  }

  fn flush(&mut self) {
    self.file.set_len(0).unwrap();
    self.file.rewind().unwrap();
    self
      .file
      .write_all(serde_json::to_string(&self.buffer).unwrap().as_bytes())
      .unwrap();
    self.file.flush().unwrap();
  }
}

#[cfg(test)]
mod test {
  use super::{Indexable, StorageItem};

  #[derive(Clone, serde::Serialize, serde::Deserialize)]
  struct TestType {
    id_: u128,
    title_: String,
  }

  impl TestType {
    fn title(&self) -> &str {
      self.title_.as_str()
    }
  }

  impl Indexable for TestType {
    fn id(&self) -> u128 {
      self.id_
    }
  }

  const STORAGE_PATH: &str = "/tmp/test.json";

  fn get_new_storage() -> StorageItem<TestType> {
    StorageItem::<TestType>::new(STORAGE_PATH)
  }

  #[test]
  fn storage_item_init() {
    let storage = get_new_storage();
    assert_eq!(storage.storage_path(), STORAGE_PATH);
  }

  #[test]
  fn storage_item_add() {
    let mut storage = get_new_storage();
    let new_item = TestType {
      id_: 10,
      title_: "Hello".to_owned(),
    };

    storage.add(new_item);
    let all_items = storage.all();

    assert_eq!(all_items.len(), 1);
  }

  #[test]
  fn storage_item_remove() {
    let mut storage = get_new_storage();
    let new_item = TestType {
      id_: 10,
      title_: "Hello".to_owned(),
    };

    storage.add(new_item);
    storage.remove(10);
    let all_items = storage.all();

    assert_eq!(all_items.is_empty(), true);
  }

  #[test]
  fn storage_item_replace() {
    let mut storage = get_new_storage();
    storage.add(TestType {
      id_: 10,
      title_: "Hello".to_owned(),
    });

    storage.replace(TestType {
      id_: 10,
      title_: "Hello, world".to_owned(),
    });

    let all_items = storage.all();
    assert_eq!(all_items.len(), 1);
    assert_eq!(all_items[0].title(), "Hello, world");
  }
}
