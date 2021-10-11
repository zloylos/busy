#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct State {
  pub last_task_id: u128,
  pub last_category_id: u128,
}
