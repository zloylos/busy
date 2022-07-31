#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct State {
  pub last_task_id: u128,
  pub last_project_id: u128,
  pub last_tag_id: u128,
}
