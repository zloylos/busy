#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum SyncerConfig {
  Empty,
  Git {
    key_file: Option<String>,
    remote: String,
    remote_branch: Option<String>,
  },
}
