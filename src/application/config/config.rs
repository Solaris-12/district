use std::{
  fs::File,
  io::{self, Read, Write},
  path::PathBuf,
};

use serde::{Deserialize, Serialize};
use serenity::all::ActivityType;

use crate::logger::{LogLevel, Logger};
use crate::{log_e, log_w, log_x};

use super::{
  auth_config::ConfigAuth,
  bots::{
    bot_config::ConfigBot, commands_config::ConfigBotCommands, presence_config::PresenceConfig,
    BotConfig,
  },
  db_config::ConfigDatabases,
  server::server_config::ConfigServer,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigApp {
  pub server_port: u16,
  pub server_address: String,
  pub auth: ConfigAuth,
  pub main_bot: BotConfig,
  pub lang_path: String,
  pub servers: Vec<ConfigServer>,
  pub databases: ConfigDatabases,
}

impl ConfigApp {
  pub fn create() -> Self {
    ConfigApp {
      server_port: 9005,
      server_address: String::from("0.0.0.0"),
      auth: ConfigAuth {
        db: String::new(),
        log: String::new(),
        ws: String::new(),
      },
      lang_path: String::from("./lang.json"),
      main_bot: BotConfig::ConfigBot(ConfigBot {
        token: String::new(),
        active_guild_id: 0,
        commands: ConfigBotCommands {
          info_command: None,
          db_search: None,
          send_command: None,
        },
        default_presence: Some(PresenceConfig {
          status: String::from("dnd"),
          kind: ActivityType::Streaming.into(),
          name: String::from("DISTRICT SERVER"),
          url: String::from("https://oxydien.dev"),
        }),
      }),
      servers: vec![],
      databases: ConfigDatabases {
        player_db_auto_clear_normal: None,
        player_db_auto_clear_strict: None,
        leaderboards: false,
      },
    }
  }

  pub fn save_to_json(&self, filename: PathBuf) -> io::Result<()> {
    let json_string = serde_json::to_string_pretty(&self)?;
    let mut file = File::create(filename)?;
    file.write_all(json_string.as_bytes())?;
    Ok(())
  }

  pub fn load_from_json(filename: PathBuf) -> io::Result<Self> {
    let mut file = File::open(filename)?;
    let mut json_string = String::new();
    file.read_to_string(&mut json_string)?;
    let config: ConfigApp = match serde_json::from_str(&json_string) {
      Ok(val) => val,
      Err(err) => {
        log_e!(format!("Error while loading config: {}", err));
        log_w!(
          "Please don't report this as issue on Github if you're not 101% sure this is our issue."
        );
        return Err(io::Error::new(io::ErrorKind::InvalidData, err));
      }
    };
    Ok(config)
  }
}
