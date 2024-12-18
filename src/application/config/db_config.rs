use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigDatabases {
  pub player_db_auto_clear_normal: Option<u32>,
  pub player_db_auto_clear_strict: Option<u32>,
  pub leaderboards: bool,
}
