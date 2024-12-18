use bot_config::ConfigBot;
use serde::{Deserialize, Serialize};

use super::server::server_bot_config::ServerBotConfig;

pub mod bot_config;
pub mod commands_config;
pub mod presence_config;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum BotConfig {
  ConfigBot(ConfigBot),
  ServerBotConfig(ServerBotConfig),
}

impl BotConfig {
  pub fn get_token(&self) -> String {
    match self {
      BotConfig::ConfigBot(cfg) => cfg.token.clone(),
      BotConfig::ServerBotConfig(cfg) => cfg.token.clone(),
    }
  }

  pub fn get_operational_guild_id(&self) -> u64 {
    match self {
      BotConfig::ConfigBot(cfg) => cfg.active_guild_id.clone(),
      BotConfig::ServerBotConfig(cfg) => cfg.active_guild_id.clone(),
    }
  }
}
