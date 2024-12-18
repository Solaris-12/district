use std::{fs, path::PathBuf};

use crate::application::config::db_config::ConfigDatabases;
use crate::logger::{LogLevel, Logger};
use crate::{log_e, log_x};

use super::leaderboards::LeaderboardDatabase;
use super::{
  database::DatabaseOperations as _, player::PlayerDatabase, punishments::PunishmentDatabase,
};

#[derive(Debug, Clone)]
pub struct DatabaseHandler {
  pub player_database: PlayerDatabase,
  pub punishment_database: PunishmentDatabase,
  pub leaderboard_database: Option<LeaderboardDatabase>,
}

impl DatabaseHandler {
  pub fn create(cfg: ConfigDatabases) -> Self {
    if let Err(e) = fs::create_dir_all(PathBuf::from("./db")) {
      log_e!(format!("Couldn't create db folder: {}", e));
      panic!("Couldn't create db folder: {}", e);
    }

    DatabaseHandler {
      player_database: PlayerDatabase::setup("./db/players.db").unwrap(),
      punishment_database: PunishmentDatabase::setup("./db/punishments.db").unwrap(),
      leaderboard_database: if cfg.leaderboards {
        Some(LeaderboardDatabase::setup("./db/leaderboards.db").unwrap())
      } else {
        None
      },
    }
  }
}
