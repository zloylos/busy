pub trait Indexable {
  fn id(&self) -> &uuid::Uuid;
}
