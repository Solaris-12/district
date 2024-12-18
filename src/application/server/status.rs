use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistrictServerStatus {
  pub open: bool,
  pub tps: u8,
  pub max_tps: u8,
  pub player_ids: Option<Vec<u64>>,
  pub duration_since_last: Option<f32>,
  pub player_count: u16,
  pub max_player_count: u16,
  pub(crate) last_heard: Option<i64>,
}

impl DistrictServerStatus {
  pub fn new() -> Self {
    DistrictServerStatus {
      open: false,
      tps: 0,
      max_tps: 0,
      player_ids: None,
      duration_since_last: Some(0f32),
      player_count: 0,
      max_player_count: 0,
      last_heard: None,
    }
  }
}
