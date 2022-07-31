use log::debug;

pub struct GitSyncer {
  main_folder_path: String,
  remote: Option<String>,
  branch: String,
}

impl GitSyncer {
  pub fn new(main_folder_path: &str, remote: Option<String>, branch: Option<String>) -> Self {
    let mut obj = Self {
      main_folder_path: main_folder_path.to_owned(),
      remote,
      branch: branch.unwrap_or("master".to_owned()),
    };
    obj.init();
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
    self.git_with_args(&["init"]);
    self.set_remote();
    self.pull();

    self.git_with_args(&["add", "-A"]);
    self.commit("initial");
  }

  pub fn commit(&mut self, msg: &str) {
    self.git_with_args(&["commit", "-a", "-m", msg]);
  }

  pub fn sync(&mut self) {
    self.pull();
    self.push();
  }

  fn push(&mut self) {
    self.git_with_args(&[
      "push",
      "--set-upstream",
      "origin",
      self.branch.clone().as_str(),
    ]);
  }

  fn pull(&mut self) {
    self.git_with_args(&["pull", "origin", self.branch.clone().as_str()]);
  }

  fn set_remote(&mut self) {
    if self.remote.is_some() {
      self.git_with_args(&[
        "remote",
        "add",
        "origin",
        self.remote.clone().unwrap().as_str(),
      ]);
    }
  }

  fn git_with_args(&mut self, args: &[&str]) {
    git_with_args(&self.main_folder_path, args);
  }
}

fn git_with_args(cwd: &str, args: &[&str]) {
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
