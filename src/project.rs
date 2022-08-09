use crate::traits::Indexable;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Project {
  id: uuid::Uuid,
  name: String,
}

impl Indexable for Project {
  fn id(&self) -> uuid::Uuid {
    self.id
  }
}

impl Project {
  pub fn new(name: &str) -> Self {
    Self {
      id: uuid::Uuid::new_v4(),
      name: name.to_owned(),
    }
  }

  pub fn name(&self) -> &str {
    self.name.as_str()
  }
}
