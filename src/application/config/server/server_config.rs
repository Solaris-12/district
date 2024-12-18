use serde::{Deserialize, Serialize};

use crate::application::config::bots::BotConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigServer {
  pub id: u64,
  pub name: String,
  pub channel_id: String,
  pub bot: BotConfig,
}
