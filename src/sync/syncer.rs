pub trait Syncer {
  fn commit(&mut self, msg: &str) -> std::io::Result<String>;
  fn sync(&mut self) -> std::io::Result<String>;
  fn push_force(&mut self) -> std::io::Result<String>;
  fn pull_force(&mut self) -> std::io::Result<String>;
}
