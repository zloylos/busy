use crate::traits::Indexable;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tag {
  id: uuid::Uuid,
  name: String,
}

impl Indexable for Tag {
  fn id(&self) -> uuid::Uuid {
    self.id
  }
}

impl Tag {
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
