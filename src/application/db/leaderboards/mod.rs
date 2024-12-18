pub(crate) mod structs;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};

use self::structs::{LeaderboardRecord, LeaderboardRecordType};

use super::database::DatabaseOperations;
use crate::logger::{LogLevel, Logger};
use crate::{log_d, log_e, log_x};

#[derive(Debug)]
pub struct LeaderboardDatabase {
  path: String,
  conn: Connection,
}

impl Clone for LeaderboardDatabase {
  fn clone(&self) -> Self {
    LeaderboardDatabase::setup(&self.path).unwrap()
  }
}

// MARK: (!) Init Leaderboard Db
impl DatabaseOperations for LeaderboardDatabase {
  fn setup(db_path: &str) -> Result<Self> {
    log_d!("Starting 'Leaderboard' database");
    let conn = match Connection::open(db_path) {
      Ok(val) => val,
      Err(e) => {
        log_e!(format!(
          "Database 'Leaderboard' threw error while opening: {}",
          e
        ));
        return Err(e);
      }
    };

    if let Err(e) = conn.execute(
      r"CREATE TABLE IF NOT EXISTS Leaderboard (
              player_id INT,
              type TINYINT,
              value FLOAT,
              date_time DATETIME
            );",
      (),
    ) {
      log_e!(format!(
        "Database 'Leaderboard' threw error while creating table 'Leaderboard': {}",
        e
      ));
      return Err(e);
    }

    Ok(Self {
      path: db_path.to_string(),
      conn,
    })
  }
}

// MARK: (!) Impl Leaderboard Db
impl LeaderboardDatabase {
  // MARK: Get all data
  pub fn get_all_data(&self) -> Result<Vec<LeaderboardRecord>> {
    let mut stmt = self.conn.prepare("SELECT * FROM Leaderboard")?;
    let data_iter = stmt.query_map([], |row| Ok(LeaderboardRecord::from_row(row).unwrap()))?;

    let data: Result<Vec<_>, _> = data_iter.collect();
    data
  }

  // MARK: Get all by type
  pub fn get_all_by_type(&self, kind: LeaderboardRecordType) -> Result<Vec<LeaderboardRecord>> {
    let mut stmt = self
      .conn
      .prepare("SELECT * FROM Leaderboard WHERE type = ?1")?;
    let data_iter = stmt.query_map([kind], |row| Ok(LeaderboardRecord::from_row(row).unwrap()))?;

    let data: Result<Vec<_>, _> = data_iter.collect();
    data
  }

  // MARK: Get all from player
  pub fn get_all_from_player(&self, player_id: u64) -> Result<Vec<LeaderboardRecord>> {
    let mut stmt = self
      .conn
      .prepare("SELECT * FROM Leaderboard WHERE player_id = ?1")?;
    let data_iter = stmt.query_map([player_id], |row| {
      Ok(LeaderboardRecord::from_row(row).unwrap())
    })?;

    let data: Result<Vec<_>, _> = data_iter.collect();
    data
  }

  // MARK: Get all from player by type
  pub fn get_all_from_player_by_type(
    &self,
    player_id: u64,
    kind: LeaderboardRecordType,
  ) -> Result<Vec<LeaderboardRecord>> {
    let mut stmt = self
      .conn
      .prepare("SELECT * FROM Leaderboard WHERE player_id = ?1 AND type = ?2")?;
    let data_iter = stmt.query_map(params! {player_id, kind}, |row| {
      Ok(LeaderboardRecord::from_row(row).unwrap())
    })?;

    let data: Result<Vec<_>, _> = data_iter.collect();
    data
  }

  // MARK: Add stat to player
  pub fn add_stat_to_player(
    &self,
    player_id: u64,
    kind: LeaderboardRecordType,
    value: f64,
    date_time: DateTime<Utc>,
  ) -> Result<()> {
    let parsed_date = date_time.to_rfc3339();

    self.conn.execute(
      "INSERT INTO Leaderboard (`player_id`, `type`, `value`, `date_time`)\
                        VALUES (?1, ?2, ?3, ?4)",
      (player_id, kind, value, parsed_date),
    )?;
    Ok(())
  }

  // MARK: Remove from player by date_time
  pub fn remove_from_player_by_date(&self, player_id: u64, date_time: DateTime<Utc>) -> Result<()> {
    self.conn.execute(
      "DELETE FROM Leaderboard WHERE player_id = ?1 AND date_time = ?2",
      params! {player_id, date_time.to_rfc3339()},
    )?;
    Ok(())
  }

  // MARK: Clear all from player
  pub fn clear_all_from_player(&self, player_id: u64) -> Result<()> {
    self.conn.execute(
      "DELETE FROM Leaderboard WHERE player_id = ?1",
      params! {player_id},
    )?;
    Ok(())
  }

  // MARK: Clear all from player by type
  pub fn clear_all_from_player_by_type(
    &self,
    player_id: u64,
    kind: LeaderboardRecordType,
  ) -> Result<()> {
    self.conn.execute(
      "DELETE FROM Leaderboard WHERE player_id = ?1 AND type = ?2",
      params! {player_id, kind},
    )?;
    Ok(())
  }
}
