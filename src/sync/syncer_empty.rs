use super::syncer::Syncer;

pub struct EmptySyncer {}

impl EmptySyncer {
  pub fn new() -> Self {
    return Self {};
  }
}

impl Syncer for EmptySyncer {
  fn commit(&mut self, msg: &str) -> std::io::Result<String> {
    return std::io::Result::Ok(format!("cmd: 'commit', msg: {msg}"));
  }
  fn sync(&mut self) -> std::io::Result<String> {
    return std::io::Result::Ok(format!("cmd: 'sync'"));
  }
  fn push_force(&mut self) -> std::io::Result<String> {
    return std::io::Result::Ok(format!("cmd: 'push_force'"));
  }
  fn pull_force(&mut self) -> std::io::Result<String> {
    return std::io::Result::Ok(format!("cmd: 'pull_force'"));
  }
}
