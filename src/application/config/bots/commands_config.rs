use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigBotCommands {
  pub info_command: Option<u64>,
  pub db_search: Option<u64>,
  pub send_command: Option<u64>,
}
