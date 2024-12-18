use super::{commands_config::ConfigBotCommands, presence_config::PresenceConfig};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigBot {
  pub token: String,
  pub active_guild_id: u64,
  pub default_presence: Option<PresenceConfig>,
  pub commands: ConfigBotCommands,
}
