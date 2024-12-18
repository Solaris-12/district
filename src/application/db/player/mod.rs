pub(crate) mod structs;

use chrono::{DateTime, Utc};
use rand::Rng as _;
use rusqlite::{params, Connection, OptionalExtension as _, Result};

use crate::logger::{LogLevel, Logger};
use crate::{log_d, log_e, log_x};

use self::structs::{
  DatabaseModifyPlayerVerification, DatabasePlayer, DatabasePlayerCount, DatabasePlayerJoin,
  DatabasePlayerVerification, PlayerVerification,
};

use super::database::DatabaseOperations;

#[derive(Debug)]
pub struct PlayerDatabase {
  path: String,
  conn: Connection,
}

impl Clone for PlayerDatabase {
  fn clone(&self) -> Self {
    PlayerDatabase::setup(&self.path).unwrap()
  }
}

// MARK: (!) Init Player Db
impl DatabaseOperations for PlayerDatabase {
  fn setup(db_path: &str) -> Result<Self> {
    log_d!("Starting 'Player' database");
    let conn = match Connection::open(db_path) {
      Ok(val) => val,
      Err(e) => {
        log_e!(format!(
          "Database 'Player' threw error while opening: {}",
          e
        ));
        return Err(e);
      }
    };

    if let Err(e) = conn.execute(
      r"CREATE TABLE IF NOT EXISTS Player (
                    player_id INT PRIMARY KEY,
                    steam_id VARCHAR(255),
                    usernames TEXT,
                    ips TEXT,
                    first_join_date DATETIME,
                    times_joined INT,
                    last_join_date DATETIME,
                    hours_played FLOAT,
                    verification_key VARCHAR(20),
                    verified_status TINYINT,
                    verified_date DATETIME,
                    discord_id TEXT,
                    do_not_track INT,
                    ban_ids TEXT,
                    rank_id INT,
                    supporter_id INT,
                    email_address VARCHAR(255)
                );",
      (),
    ) {
      log_e!(format!(
        "Database 'Player' threw error while creating table 'Player': {}",
        e
      ));
      return Err(e);
    }

    if let Err(e) = conn.execute(
      r"CREATE TABLE IF NOT EXISTS PlayerCount (
                timestamp INT PRIMARY KEY,
                player_count INT,
                server_id INT
            );",
      (),
    ) {
      log_e!(format!(
        "Database 'Player' threw error while creating table 'PlayerCount': {}",
        e
      ));
      return Err(e);
    }

    Ok(PlayerDatabase {
      path: db_path.to_string(),
      conn,
    })
  }
}

// MARK: (!) Impl Player Db
impl PlayerDatabase {
  // MARK: Add player
  pub fn add_player(
    &self,
    steam_id: String,
    username: String,
    ip_addr: String,
    do_not_track: bool,
    first_join: DateTime<Utc>,
  ) -> Result<()> {
    let mut rng = rand::thread_rng();
    let id: u64 = rng.gen_range(3202036800000000..=3923372036854775807);
    let usernames = format!("[\"{}\"]", username);
    let ips = format!("[\"{}\"]", ip_addr);
    let joined_date = first_join.to_rfc3339();

    self.conn.execute("INSERT INTO Player (`player_id`, `steam_id`, `usernames`, `ips`, `first_join_date`, `times_joined`, `last_join_date`, `hours_played`, `verified_status`, `ban_ids`, `do_not_track`)\
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                   (id, steam_id, usernames, ips, &joined_date, 1, &joined_date, 0, PlayerVerification::None, "[]", &do_not_track))?;
    Ok(())
  }

  /// Gets player in db by its player_id
  /// returns [`Result`] with [`DatabasePlayer`] if found
  /// throws [`rusqlite::Error::QueryReturnedNoRows`] when not found
  // MARK: Get player by id
  pub fn get_player_by_id(&self, id: u64) -> Result<DatabasePlayer> {
    let mut stmt = self
      .conn
      .prepare("SELECT * FROM Player WHERE `player_id` = ?1")?;
    let person_iter = stmt.query_map([id], |row| Ok(DatabasePlayer::from_row(row).unwrap()))?;

    let mut players = person_iter;
    match players.next() {
      Some(Ok(player)) => Ok(player),
      _ => Err(rusqlite::Error::QueryReturnedNoRows),
    }
  }

  /// Gets players in db by their steam_id
  /// returns [`Result`] with [`std::Vec`] containing [`DatabasePlayer`]
  // MARK: Get all players
  pub fn get_all_players(&self) -> Result<Vec<DatabasePlayer>> {
    let mut stmt = self.conn.prepare("SELECT * FROM Player")?;
    let mut person_iter = stmt.query_map([], |row| Ok(DatabasePlayer::from_row(row).unwrap()))?;

    let mut players = Vec::new();
    while let Some(result) = person_iter.next() {
      match result {
        Ok(player) => players.push(player),
        Err(e) => return Err(e),
      }
    }

    Ok(players)
  }

  /// Gets players in db by their steam_id
  /// returns [`Result`] with [`std::Vec`] containing [`DatabasePlayer`]
  // MARK: Get player by steam ID
  pub fn get_players_by_steam(&self, steam_id: &str) -> Result<Vec<DatabasePlayer>> {
    let mut stmt = self
      .conn
      .prepare("SELECT * FROM Player WHERE `steam_id` = ?1")?;
    let mut person_iter =
      stmt.query_map([steam_id], |row| Ok(DatabasePlayer::from_row(row).unwrap()))?;

    let mut players = Vec::new();
    while let Some(result) = person_iter.next() {
      match result {
        Ok(player) => players.push(player),
        Err(e) => return Err(e),
      }
    }

    Ok(players)
  }

  /// Gets verified players in db by their discord_id
  /// returns [`Result`] with [`std::Vec`] containing [`DatabasePlayer`]
  // MARK: Get player by discord ID
  pub fn get_players_by_discord(&self, discord_id: &str) -> Result<Vec<DatabasePlayer>> {
    let mut stmt = self
      .conn
      .prepare("SELECT * FROM Player WHERE `discord_id` = ?1")?;
    let mut person_iter = stmt.query_map([discord_id], |row| {
      Ok(DatabasePlayer::from_row(row).unwrap())
    })?;

    let mut players = Vec::new();
    while let Some(result) = person_iter.next() {
      match result {
        Ok(player) => players.push(player),
        Err(e) => return Err(e),
      }
    }

    Ok(players)
  }

  pub fn get_player_by_discord_or_steam(
    &mut self,
    discord_id: &String,
    steam_id: &String,
  ) -> Result<Option<DatabasePlayer>> {
    let tx = self.conn.transaction()?;

    let query = r#"
            SELECT * FROM Player
            WHERE discord_id = ?1 OR steam_id = ?2
        "#;

    let player: Option<DatabasePlayer> = tx
      .query_row(query, params![discord_id, steam_id], |row| {
        DatabasePlayer::from_row(row)
      })
      .optional()?;

    Ok(player)
  }

  /// Updates the player based on player_id
  // MARK: Modify player
  pub fn modify_player(&mut self, id: u64, data: DatabasePlayer) -> Result<(), String> {
    let tx = self.conn.transaction().map_err(|e| e.to_string())?;

    let update_query = r#"
            UPDATE Player
            SET steam_id = ?1,
                usernames = ?2,
                ips = ?3,
                first_join_date = ?4,
                times_joined = ?5,
                last_join_date = ?6,
                hours_played = ?7,
                verification_key = ?8,
                verified_status = ?9,
                verified_date = ?10,
                discord_id = ?11,
                ban_ids = ?12,
                rank_id = ?13,
                do_not_track = ?14,
                supporter_id = ?15,
                email_address = ?16
            WHERE player_id = ?17
        "#;

    let escaped_usernames: Vec<String> = data
      .usernames
      .iter()
      .map(|username| format!("'{}'", username.replace("'", "")))
      .collect();

    let escaped_ips: Vec<String> = data
      .ips
      .iter()
      .map(|ip| format!("'{}'", ip.replace("'", "")))
      .collect();

    let escaped_ban_ids: Vec<String> = data
      .ban_ids
      .unwrap_or(vec![])
      .iter()
      .map(ToString::to_string)
      .collect();

    tx.execute(
      update_query,
      params![
        data.steam_id,
        serde_json::to_string(&escaped_usernames).map_err(|e| e.to_string())?,
        serde_json::to_string(&escaped_ips).map_err(|e| e.to_string())?,
        data.first_join_date.to_rfc3339(),
        data.times_joined,
        data.last_join_date.to_rfc3339(),
        data.hours_played,
        data.verification_key,
        data.verified_status,
        data
          .verified_date
          .map_or("NULL".to_string(), |date| date.to_rfc3339()),
        data.discord_id,
        serde_json::to_string(&escaped_ban_ids).map_err(|e| e.to_string())?,
        data.rank_id,
        data.do_not_track,
        data.supporter_id,
        data.email_address,
        id
      ],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
  }

  // MARK: Remove InActive Players
  pub fn remove_inactive_players(
    &mut self,
    days_inactive: u32,
    do_not_track_only: bool,
  ) -> Result<()> {
    let cutoff_date = Utc::now() - chrono::Duration::days(days_inactive as i64);
    let cutoff_date_str = cutoff_date.to_rfc3339();

    let delete_query = format!(
      r#"
            DELETE FROM Player
            WHERE last_join_date < ?1 AND (verified_status != 3 AND verified_status != 4) {}
        "#,
      if do_not_track_only {
        "AND do_not_track = 1"
      } else {
        ""
      }
    );

    self.conn.execute(&delete_query, params![cutoff_date_str])?;

    Ok(())
  }

  // MARK: Add play time
  pub fn add_playtime_to_player(&mut self, player_id: u64, amount: f32) -> Result<()> {
    let mut player = self.get_player_by_id(player_id)?;
    player.hours_played += amount;
    self
      .modify_player(player_id, player)
      .map_err(|_| rusqlite::Error::ExecuteReturnedResults)
  }

  // MARK: Verifications
  pub fn get_player_verification(&self, player_id: u64) -> Result<DatabasePlayerVerification> {
    let player = self.get_player_by_id(player_id)?;
    let verification = DatabasePlayerVerification::from(player);
    Ok(verification)
  }

  pub fn add_player_verification(&mut self, data: DatabaseModifyPlayerVerification) -> Result<()> {
    let players = self.get_players_by_steam(&data.steam_id)?;

    if players.is_empty() {
      return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    let player_db = &players[0];
    let player_verified_status = player_db
      .verified_status
      .unwrap_or_else(|| PlayerVerification::None);
    if player_verified_status != PlayerVerification::None {
      return Err(rusqlite::Error::InvalidQuery);
    }

    self.set_player_verification(
      player_db.player_id,
      PlayerVerification::Created,
      data.discord_id,
      data.code,
    )
  }

  pub fn set_player_verification(
    &mut self,
    player_id: u64,
    verified_status: PlayerVerification,
    discord_id: Option<String>,
    code: Option<String>,
  ) -> Result<()> {
    let player = self.get_player_by_id(player_id)?;
    let current_date = Utc::now();

    let tx = self.conn.transaction()?;

    let update_query = r#"
                UPDATE Player
                SET
                    verification_key = ?1,
                    verified_status = ?2,
                    verified_date = ?3,
                    discord_id = ?4
                WHERE player_id = ?5
            "#;

    let verification_key = code.unwrap_or(
      player
        .verification_key
        .unwrap_or_else(|| "NULL".to_string()),
    );
    let discord_id = discord_id.unwrap_or(player.discord_id.unwrap_or_else(|| "NULL".to_string()));

    tx.execute(
      update_query,
      params![
        verification_key,
        verified_status,
        current_date.to_rfc3339(),
        discord_id,
        player_id
      ],
    )?;

    tx.commit()?;
    Ok(())
  }

  // MARK: Player count
  pub fn get_player_count(&self) -> Result<Vec<DatabasePlayerCount>> {
    let mut stmt = self.conn.prepare("SELECT * FROM PlayerCount")?;
    let mut person_iter =
      stmt.query_map([], |row| Ok(DatabasePlayerCount::from_row(row).unwrap()))?;

    let mut players = Vec::new();
    while let Some(result) = person_iter.next() {
      match result {
        Ok(player) => players.push(player),
        Err(e) => return Err(e),
      }
    }

    Ok(players)
  }

  pub fn get_player_count_from(&self, from_timestamp: u64) -> Result<Vec<DatabasePlayerCount>> {
    let mut stmt = self
      .conn
      .prepare("SELECT * FROM PlayerCount WHERE timestamp >= ?1")?;
    let mut person_iter = stmt.query_map([from_timestamp], |row| {
      Ok(DatabasePlayerCount::from_row(row).unwrap())
    })?;

    let mut players = Vec::new();
    while let Some(result) = person_iter.next() {
      match result {
        Ok(player) => players.push(player),
        Err(e) => return Err(e),
      }
    }

    Ok(players)
  }

  pub fn set_player_count_auto(&self, player_count: u32) -> Result<()> {
    let current_timestamp = Utc::now().timestamp() as u64;

    self.conn.execute(
      "INSERT INTO PlayerCount (timestamp, player_count) VALUES (?1, ?2)",
      params![current_timestamp, player_count],
    )?;

    Ok(())
  }

  pub fn set_player_count(&self, count: DatabasePlayerCount) -> Result<()> {
    self.conn.execute(
      "INSERT INTO PlayerCount (timestamp, player_count) VALUES (?1, ?2)",
      params![count.timestamp, count.player_count],
    )?;

    Ok(())
  }

  pub fn player_joined(&mut self, data: DatabasePlayerJoin) -> Result<DatabasePlayer> {
    let mut existing_records = self.get_players_by_steam(&data.steam_id)?;

    if existing_records.len() > 0 {
      let mut player = existing_records[0].clone();

      if !player.usernames.contains(&data.username) {
        player.usernames.push(data.username.clone());
      }

      if !player.ips.contains(&data.ip_addr) {
        player.ips.push(data.ip_addr.clone());
      }

      player.do_not_track = data.do_not_track;
      player.times_joined += 1;
      player.last_join_date = Utc::now();
      let _ = self
        .modify_player(player.player_id, player.clone())
        .map_err(|_| rusqlite::Error::ExecuteReturnedResults)?;
      return Ok(player);
    }
    self.add_player(
      data.steam_id.clone(),
      data.username.clone(),
      data.ip_addr.clone(),
      data.do_not_track.clone(),
      Utc::now(),
    )?;

    existing_records = self.get_players_by_steam(&data.steam_id)?;
    if existing_records.is_empty() {
      return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(existing_records[0].clone())
  }
}
