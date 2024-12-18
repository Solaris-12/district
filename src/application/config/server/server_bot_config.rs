use serde::{Deserialize, Serialize};

use crate::application::config::bots::{
  commands_config::ConfigBotCommands, presence_config::PresenceConfig,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerBotConfig {
  pub token: String,
  pub active_guild_id: u64,
  pub use_presence: Option<bool>,
  pub default_presence: Option<PresenceConfig>,
  pub active_presence: Option<PresenceConfig>,
  pub commands: ConfigBotCommands,
}
