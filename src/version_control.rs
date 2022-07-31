use log::debug;

pub struct VersionControl {
  main_folder_path: String,
}

impl VersionControl {
  pub fn new(main_folder_path: &str) -> Self {
    let mut obj = Self {
      main_folder_path: main_folder_path.to_owned(),
    };
    let _ = obj.init();
    return obj;
  }

  fn init(&mut self) {
    if std::path::Path::new(&self.main_folder_path)
      .join(".git")
      .exists()
    {
      self.pull();
      return;
    }
    self.run_with_args(&["init"]);
    self.run_with_args(&["add", "-A"]);
    self.commit("initial");
  }

  pub fn commit(&mut self, msg: &str) {
    self.run_with_args(&["commit", "-a", "-m", msg]);
    self.push();
  }

  fn push(&mut self) {
    self.run_with_args(&["push"]);
  }

  fn pull(&mut self) {
    self.run_with_args(&["pull"]);
  }

  fn run_with_args(&mut self, args: &[&str]) {
    let output = std::process::Command::new("git")
      .current_dir(&self.main_folder_path)
      .args(args)
      .output()
      .unwrap();

    if !output.status.success() {
      debug!(
        "git with args: {:?} err: {} status: {}",
        args,
        String::from_utf8(output.stderr.clone()).unwrap_or_default(),
        output.status
      );
      return;
    }

    debug!(
      "git with args: {:?} output: {:?} status: {}",
      args,
      String::from_utf8(output.stdout.clone()).unwrap_or_default(),
      output.status
    );
  }
}
