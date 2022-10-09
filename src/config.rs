use crate::sync::SyncerConfig;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
  pub storage_dir_path: String,
  pub syncer: SyncerConfig,
}

impl Config {
  pub fn new() -> Self {
    const BUSY_DEFAULT_STORAGE_DIR: &str = ".busy";
    const BUSY_DEFAULT_CONFIG_NAME: &str = ".busy.json";

    let home_env = std::env::var("HOME").unwrap();
    let home = std::path::Path::new(home_env.as_str());

    let config_file_path = match std::env::var("BUSY_CONFIG") {
      Ok(file_path) => std::path::Path::new(&file_path).to_path_buf(),
      Err(_) => home.join(BUSY_DEFAULT_CONFIG_NAME),
    };

    let get_config_file = || {
      std::fs::File::options()
        .create(true)
        .write(true)
        .read(true)
        .open(config_file_path.clone())
        .unwrap()
    };

    if !config_file_path.exists() {
      let config = Self {
        storage_dir_path: home
          .join(BUSY_DEFAULT_STORAGE_DIR)
          .to_str()
          .unwrap()
          .to_owned(),
        syncer: SyncerConfig::Git {
          remote: "git@github.com:zloylos/pomidorka_db.git".to_owned(),
          remote_branch: Some("master".to_owned()),
        },
      };

      serde_json::to_writer_pretty(get_config_file(), &config).unwrap();
      return config;
    }

    return serde_json::from_reader(get_config_file()).unwrap();
  }
}
