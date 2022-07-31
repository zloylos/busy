use crate::traits::Indexable;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tag {
  id_: u128,
  name_: String,
}

impl Indexable for Tag {
  fn id(&self) -> u128 {
    self.id_
  }
}

impl Tag {
  pub fn new(id: u128, name: &str) -> Self {
    Self {
      id_: id,
      name_: name.to_owned(),
    }
  }

  pub fn name(&self) -> &str {
    self.name_.as_str()
  }
}
