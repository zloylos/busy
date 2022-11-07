use crate::{project::Project, tag::Tag, task::Task};

pub trait Storage {
  fn shorten_id(&self, id: uuid::Uuid) -> String;
  fn resolve_id(&self, id: &str) -> Option<uuid::Uuid>;

  fn tasks(&self) -> Vec<Task>;
  fn add_task(&mut self, task: &Task);
  fn remove_task(&mut self, task_id: uuid::Uuid) -> Result<(), String>;
  fn replace_task(&mut self, task: &Task) -> Result<(), String>;
  fn replace_tasks(&mut self, tasks: Vec<Task>);

  fn add_tag(&mut self, tag: &Tag);
  fn replace_tag(&mut self, tag: &Tag) -> Result<(), String>;
  fn replace_tags(&mut self, tags: Vec<Tag>);
  fn tags(&self) -> Vec<Tag>;
  fn tag_by_id(&self, id: uuid::Uuid) -> Option<&Tag>;
  fn find_tag_by_name(&self, tag_name: &str) -> Option<Tag>;
  fn find_tag_by_names(&self, tag_names: &Vec<String>) -> Vec<Tag>;
  fn find_tags(&self, tag_ids: &Vec<uuid::Uuid>) -> Vec<Tag>;

  fn add_project(&mut self, project: &Project);
  fn replace_project(&mut self, project: &Project) -> Result<(), String>;
  fn projects(&self) -> Vec<Project>;
}
