use log::debug;

pub struct VersionControl {
  main_folder_path: String,
  remote: Option<String>,
}

impl VersionControl {
  pub fn new(main_folder_path: &str, remote: Option<String>) -> Self {
    let mut obj = Self {
      main_folder_path: main_folder_path.to_owned(),
      remote,
    };
    let _ = obj.init();
    return obj;
  }

  fn init(&mut self) {
    if std::path::Path::new(&self.main_folder_path)
      .join(".git")
      .exists()
    {
      self.set_remote();
      return;
    }
    self.run_with_args(&["init"]);
    self.set_remote();
    self.run_with_args(&["add", "-A"]);
    self.commit("initial");
  }

  pub fn commit(&mut self, msg: &str) {
    self.run_with_args(&["commit", "-a", "-m", msg]);
  }

  pub fn push(&mut self) {
    self.run_with_args(&["push", "--set-upstream", "origin", "master"]);
  }

  pub fn pull(&mut self) {
    self.run_with_args(&["pull", "origin", "master"]);
  }

  fn set_remote(&mut self) {
    if self.remote.is_some() {
      self.run_with_args(&[
        "remote",
        "add",
        "origin",
        self.remote.clone().unwrap().as_str(),
      ]);
    }
  }

  fn run_with_args(&mut self, args: &[&str]) {
    run_with_args(&self.main_folder_path, args);
  }
}

fn run_with_args(cwd: &str, args: &[&str]) {
  let output = std::process::Command::new("git")
    .current_dir(cwd)
    .args(args)
    .output()
    .unwrap();

  if !output.status.success() {
    debug!(
      "git with err: {} status: {}",
      String::from_utf8(output.stderr.clone()).unwrap_or_default(),
      output.status
    );
    return;
  }

  debug!(
    "git with output: {:?} status: {}",
    String::from_utf8(output.stdout.clone()).unwrap_or_default(),
    output.status
  );
}
