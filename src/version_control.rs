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
      self.pull();
      return;
    }
    self.run_with_args_sync(&["init"]);
    self.set_remote();
    self.run_with_args_sync(&["add", "-A"]);
    self.commit("initial");
  }

  pub fn commit(&mut self, msg: &str) {
    self.run_with_args_sync(&["commit", "-a", "-m", msg]);
    self.push();
  }

  fn set_remote(&mut self) {
    if self.remote.is_some() {
      self.run_with_args_sync(&[
        "remote",
        "add",
        "origin",
        self.remote.clone().unwrap().as_str(),
      ]);
    }
  }

  fn push(&mut self) {
    self.run_with_args_async(&["push", "--set-upstream", "origin", "master"]);
  }

  fn pull(&mut self) {
    self.run_with_args_sync(&["pull", "origin", "master"]);
  }

  fn run_with_args_sync(&mut self, args: &[&str]) {
    let child = run_with_args(self.main_folder_path.as_str(), args);
    process_child(child.unwrap());
  }

  fn run_with_args_async(&mut self, args: &[&str]) {
    let child = run_with_args(self.main_folder_path.as_str(), args);
    std::thread::spawn(move || {
      process_child(child.unwrap());
    });
  }
}

fn process_child(child: std::process::Child) {
  let output = child.wait_with_output().unwrap();
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

fn run_with_args(cwd: &str, args: &[&str]) -> std::io::Result<std::process::Child> {
  return std::process::Command::new("git")
    .current_dir(cwd)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .args(args)
    .spawn();
}
