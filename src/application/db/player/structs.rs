use chrono::{DateTime, Utc};
use rusqlite::{
    types::{FromSql, FromSqlError, ValueRef},
    Result, ToSql,
};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::application::utils;

#[repr(u8)]
#[derive(Clone, Copy, Serialize_repr, Deserialize_repr, PartialEq)]
/// # Player verification status
// MARK: (enum) Player verification
pub enum PlayerVerification {
    None = 0,      // Not verified
    Created = 1,   // When code created, but not sent
    Pending = 2,   // When sent to player
    Success = 3,   // Player is verified
    Full = 4,      // Player has newer verification
    Expired = 5,   // When player verification expired
    Banned = 6,    // When player got banned (from this feature or discord)
    Suspended = 7, // When player's verification under review
}

impl PlayerVerification {
    fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl ToSql for PlayerVerification {
    fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::from(self.as_u8()))
    }
}

impl FromSql for PlayerVerification {
    fn column_result(value: ValueRef<'_>) -> Result<PlayerVerification, FromSqlError> {
        let int_value = value.as_i64()?;
        match int_value {
            0 => Ok(PlayerVerification::None),
            1 => Ok(PlayerVerification::Created),
            2 => Ok(PlayerVerification::Pending),
            3 => Ok(PlayerVerification::Success),
            4 => Ok(PlayerVerification::Full),
            5 => Ok(PlayerVerification::Expired),
            6 => Ok(PlayerVerification::Banned),
            7 => Ok(PlayerVerification::Suspended),
            _ => Err(FromSqlError::OutOfRange(int_value)),
        }
    }
}

//

#[derive(Clone, Serialize)]
/// Player
// MARK: Player
pub struct DatabasePlayer {
    pub player_id: u64,
    pub steam_id: String,
    pub usernames: Vec<String>,
    pub ips: Vec<String>,
    pub first_join_date: DateTime<Utc>,
    pub times_joined: u32,
    pub last_join_date: DateTime<Utc>,
    pub hours_played: f32,
    pub verification_key: Option<String>,
    pub verified_status: Option<PlayerVerification>,
    pub verified_date: Option<DateTime<Utc>>,
    pub discord_id: Option<String>,
    pub ban_ids: Option<Vec<String>>,
    pub do_not_track: bool,
    pub rank_id: Option<u16>,
    pub supporter_id: Option<u32>,
    pub email_address: Option<String>,
}

impl DatabasePlayer {
    pub fn from_row(row: &rusqlite::Row) -> Result<Self> {
        let usernames =
            match utils::json::parse_and_trim_json_strings(row.get::<_, String>(2)?.as_str()) {
                Ok(val) => val,
                Err(_) => Vec::new(),
            };
        let ips = match utils::json::parse_and_trim_json_strings(row.get::<_, String>(3)?.as_str())
        {
            Ok(val) => val,
            Err(_) => Vec::new(),
        };

        let first_join_date = match utils::time::parse_rfc3339_to_utc(row.get::<_, String>(4)?) {
            Ok(val) => val,
            Err(_) => Utc::now(),
        };

        let last_join_date = match utils::time::parse_rfc3339_to_utc(row.get::<_, String>(6)?) {
            Ok(val) => val,
            Err(_) => Utc::now(),
        };

        let verified_date =
            utils::time::parse_rfc3339_to_utc_or_none(row.get::<_, Option<String>>(10)?);

        let ban_ids: Option<Vec<String>> = row
            .get::<_, Option<String>>(13)?
            .as_ref()
            .map(|s| serde_json::from_str(s).unwrap_or_default())
            .unwrap_or_default();

        Ok(DatabasePlayer {
            player_id: row.get(0)?,
            steam_id: row.get(1)?,
            usernames,
            ips,
            first_join_date,
            times_joined: row.get(5)?,
            last_join_date,
            hours_played: row.get(7)?,
            verification_key: row.get(8)?,
            verified_status: row.get(9)?,
            verified_date,
            discord_id: row.get(11)?,
            do_not_track: row.get(12)?,
            ban_ids,
            rank_id: row.get(14)?,
            supporter_id: row.get(15)?,
            email_address: row.get(16)?,
        })
    }
    pub(crate) fn is_verified(&self) -> bool {
        matches!(
            self.verified_status,
            Some(PlayerVerification::Success) | Some(PlayerVerification::Full)
        )
    }
    pub(crate) fn is_verification_banned(&self) -> bool {
        matches!(
            self.verified_status,
            Some(PlayerVerification::Banned) | Some(PlayerVerification::Suspended)
        )
    }
}

#[derive(Clone, Serialize, Deserialize)]
/// Player count
// MARK: Player count
pub struct DatabasePlayerCount {
    pub timestamp: u64,
    pub player_count: u32,
    pub server_id: u64,
}

impl DatabasePlayerCount {
    pub fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(DatabasePlayerCount {
            timestamp: row.get(0)?,
            player_count: row.get(1)?,
            server_id: row.get(2).unwrap_or(0),
        })
    }
}

#[derive(Clone, Serialize)]
/// Player Verification
// MARK: (obj) Player verification
pub struct DatabasePlayerVerification {
    pub player_id: u64,
    pub steam_id: String,
    pub verification_key: Option<String>,
    pub verified_status: Option<PlayerVerification>,
    pub verified_date: Option<DateTime<Utc>>,
    pub discord_id: Option<String>,
    pub is_considered_verified: bool,
}

impl From<DatabasePlayer> for DatabasePlayerVerification {
    fn from(player: DatabasePlayer) -> Self {
        let is_verified = player.is_verified();
        DatabasePlayerVerification {
            player_id: player.player_id,
            steam_id: player.steam_id,
            verification_key: player.verification_key,
            verified_status: player.verified_status,
            verified_date: player.verified_date,
            discord_id: player.discord_id,
            is_considered_verified: is_verified,
        }
    }
}

#[derive(Serialize, Deserialize)]
/// Player verification modification
// MARK: (obj) Player verification modification
pub struct DatabaseModifyPlayerVerification {
    pub player_id: u64,
    pub steam_id: String,
    pub discord_id: Option<String>,
    pub verified_status: PlayerVerification,
    pub code: Option<String>,
}

#[derive(Serialize, Deserialize)]
/// Player Join
// MARK: (obj) Player join
pub struct DatabasePlayerJoin {
    pub username: String,
    pub steam_id: String,
    pub ip_addr: String,
    pub do_not_track: bool,
}
