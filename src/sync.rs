use log::debug;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum SyncerConfig {
  Empty,
  Git {
    key_file: Option<String>,
    remote: String,
    remote_branch: Option<String>,
  },
}

pub trait Syncer {
  fn commit(&mut self, msg: &str) -> std::io::Result<String>;
  fn sync(&mut self) -> std::io::Result<String>;
  fn push_force(&mut self) -> std::io::Result<String>;
  fn pull_force(&mut self) -> std::io::Result<String>;
}

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

pub struct GitSyncer {
  main_folder_path: String,
  key_file: Option<String>,
  remote: Option<String>,
  branch: String,
}

impl GitSyncer {
  pub fn new(
    main_folder_path: &str,
    key_file: Option<String>,
    remote: Option<String>,
    branch: Option<String>,
  ) -> Self {
    let mut obj = Self {
      main_folder_path: main_folder_path.to_owned(),
      key_file,
      remote,
      branch: branch.unwrap_or("main".to_owned()),
    };
    _ = obj.init();
    return obj;
  }

  fn init(&mut self) -> std::io::Result<String> {
    if std::path::Path::new(&self.main_folder_path)
      .join(".git")
      .exists()
    {
      return self.set_remote();
    }
    self.git_with_args(&["init", "-b", self.branch.clone().as_str()])?;
    _ = self.set_remote();
    _ = self.pull();

    return Ok("initialization success".to_string());
  }

  fn push(&mut self) -> std::io::Result<String> {
    return self.git_with_args(&["push", "-u", "origin", self.branch.clone().as_str()]);
  }

  fn pull(&mut self) -> std::io::Result<String> {
    return self.git_with_args(&["pull", "origin", self.branch.clone().as_str()]);
  }

  fn set_remote(&mut self) -> std::io::Result<String> {
    if self.remote.is_some() {
      return match self.set_remote_url() {
        Ok(res) => Ok(res),
        Err(_) => self.git_with_args(&[
          "remote",
          "add",
          "origin",
          self.remote.clone().unwrap().as_str(),
        ]),
      };
    }
    return Ok("remote isn't set".to_string());
  }

  fn set_remote_url(&mut self) -> std::io::Result<String> {
    self.git_with_args(&[
      "remote",
      "set-url",
      "origin",
      self.remote.clone().unwrap().as_str(),
    ])
  }

  fn git_with_args(&mut self, args: &[&str]) -> std::io::Result<String> {
    return git_with_args(&self.main_folder_path, self.key_file.as_ref(), args);
  }
}

impl Syncer for GitSyncer {
  fn commit(&mut self, msg: &str) -> std::io::Result<String> {
    self.git_with_args(&["add", "-A"])?;
    return self.git_with_args(&["commit", "-a", "-m", msg]);
  }

  fn sync(&mut self) -> std::io::Result<String> {
    let pull_output = self.pull()?;
    let push_output = self.push()?;
    return Ok(format!(
      "git pull output:\n{}\n\ngit push output:\n{}",
      pull_output, push_output
    ));
  }

  fn push_force(&mut self) -> std::io::Result<String> {
    return self.git_with_args(&[
      "push",
      "--force",
      "-u",
      "origin",
      self.branch.clone().as_str(),
    ]);
  }

  fn pull_force(&mut self) -> std::io::Result<String> {
    return self.git_with_args(&[
      "pull",
      "--force",
      "--rebase",
      "origin",
      self.branch.clone().as_str(),
    ]);
  }
}

fn git_with_args(cwd: &str, key_file: Option<&String>, args: &[&str]) -> std::io::Result<String> {
  debug!("run git with args: {args:?} cwd: {cwd} key_file: {key_file:?}");

  let mut command = std::process::Command::new("git");
  command.current_dir(cwd);
  if key_file.is_some() {
    command.env("GIT_SSH_COMMAND", format!("ssh -i {}", key_file.unwrap()));
  }
  let output = command.args(args).output()?;

  let stdout = String::from_utf8(output.stdout.clone()).unwrap_or_default();
  if !output.status.success() {
    debug!("git with err: {} status: {}", stdout, output.status);
    return Err(std::io::Error::new(std::io::ErrorKind::Other, stdout));
  }

  debug!("git with output: {:?} status: {}", stdout, output.status);

  return Ok(stdout);
}
