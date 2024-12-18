pub(crate) mod structs;

use rusqlite::{Connection, Result};

use crate::logger::{LogLevel, Logger};
use crate::{log_d, log_e, log_x};

use self::structs::DatabasePunishment;

use super::database::DatabaseOperations;

#[derive(Debug)]
pub struct PunishmentDatabase {
    path: String,
    pub conn: Connection,
}

impl Clone for PunishmentDatabase {
    fn clone(&self) -> Self {
        PunishmentDatabase::setup(&self.path).unwrap()
    }
}

// MARK: (!) Init Punishment Db
impl DatabaseOperations for PunishmentDatabase {
    fn setup(db_path: &str) -> Result<Self> {
        log_d!("Starting 'Punishment' database");
        let conn = match Connection::open(db_path) {
            Ok(val) => val,
            Err(e) => {
                log_e!(format!(
                    "Database 'Punishment' threw error while opening: {}",
                    e
                ));
                return Err(e);
            }
        };

        if let Err(e) = conn.execute(
            r"CREATE TABLE IF NOT EXISTS Punishment (
                punishment_id VARCHAR(9) PRIMARY KEY NOT NULL,
                player_id INTEGER NOT NULL,
                username TEXT,
                steam_id TEXT,
                ip TEXT,
                reason TEXT,
                punishment_duration INTEGER,
                punishment_created_at DATETIME,
                issuer_steam_id TEXT,
                issuer_name TEXT,
                issuer_ip TEXT,
                punishment_type INTEGER
            );",
            (),
        ) {
            log_e!(format!(
                "Database 'Punishment' threw error while creating table 'Punishment': {}",
                e
            ));
            return Err(e);
        }

        Ok(PunishmentDatabase {
            path: db_path.to_string(),
            conn,
        })
    }
}

// MARK: (!) Impl Punishment Db
impl PunishmentDatabase {
    // MARK: Get all punishments
    pub fn get_all_punishments(&self) -> rusqlite::Result<Vec<DatabasePunishment>> {
        let mut stmt = self.conn.prepare("SELECT * FROM Punishment")?;
        let punishment_iter =
            stmt.query_map([], |row| Ok(DatabasePunishment::from_row(row).unwrap()))?;

        let punishments: Result<Vec<DatabasePunishment>, rusqlite::Error> =
            punishment_iter.collect();
        punishments
    }

    // MARK: Get punishment by punishment ID
    pub fn get_punishment_by_punishment_id(
        &self,
        punishment_id: String,
    ) -> rusqlite::Result<DatabasePunishment> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM Punishment WHERE punishment_id = ?1")?;
        let punishment_iter = stmt.query_map([punishment_id], |row| {
            Ok(DatabasePunishment::from_row(row).unwrap())
        })?;

        let mut punishments = punishment_iter;
        match punishments.next() {
            Some(Ok(punishment)) => Ok(punishment),
            _ => Err(rusqlite::Error::QueryReturnedNoRows),
        }
    }

    // MARK: Get punishments by player ID
    pub fn get_punishments_by_player_id(
        &self,
        player_id: u64,
    ) -> rusqlite::Result<Vec<DatabasePunishment>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM Punishment WHERE player_id = ?1")?;
        let punishment_iter = stmt.query_map([player_id], |row| {
            Ok(DatabasePunishment::from_row(row).unwrap())
        })?;

        let punishments: Result<Vec<DatabasePunishment>, rusqlite::Error> =
            punishment_iter.collect();
        punishments
    }

    // MARK: Get punishments by steam ID
    pub fn get_punishments_by_steam_id(
        &self,
        steam_id: String,
    ) -> rusqlite::Result<Vec<DatabasePunishment>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM Punishment WHERE steam_id = ?1")?;
        let punishment_iter = stmt.query_map([steam_id], |row| {
            Ok(DatabasePunishment::from_row(row).unwrap())
        })?;

        let punishments: Result<Vec<DatabasePunishment>, rusqlite::Error> =
            punishment_iter.collect();
        punishments
    }

    // MARK: Get punishments by player IP
    pub fn get_punishments_by_ip(&self, ip: String) -> rusqlite::Result<Vec<DatabasePunishment>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM Punishment WHERE ip = ?1")?;
        let punishment_iter =
            stmt.query_map([ip], |row| Ok(DatabasePunishment::from_row(row).unwrap()))?;

        let punishments: Result<Vec<DatabasePunishment>, rusqlite::Error> =
            punishment_iter.collect();
        punishments
    }

    // MARK: Get punishments made by steam ID
    pub fn get_punishments_from_steam_id(
        &self,
        steam_id: String,
    ) -> rusqlite::Result<Vec<DatabasePunishment>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM Punishment WHERE issuer_steam_id = ?1")?;
        let punishment_iter = stmt.query_map([steam_id], |row| {
            Ok(DatabasePunishment::from_row(row).unwrap())
        })?;

        let punishments: Result<Vec<DatabasePunishment>, rusqlite::Error> =
            punishment_iter.collect();
        punishments
    }

    // MARK: New punishment
    pub fn create_new_punishment(&self, data: &DatabasePunishment) -> rusqlite::Result<()> {
        self.conn.execute("INSERT INTO Punishment (`punishment_id`, `player_id`, `username`, `steam_id`, `ip`, `reason`, `punishment_duration`, `punishment_created_at`, `issuer_steam_id`, `issuer_name`, `issuer_ip`, `punishment_type`)\
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
      [
          &data.punishment_id,
          &data.player_id.to_string(),
          &data.username,
          &data.steam_id,
          &data.ip,
          &data.reason,
          &data.punishment_duration.to_string(),
          &data.punishment_created_at.to_rfc3339(),
          &data.issuer_steam_id,
          &data.issuer_name,
          &data.issuer_ip,
          &data.punishment_type.as_u8().to_string(),
      ])?;

        Ok(())
    }
}
