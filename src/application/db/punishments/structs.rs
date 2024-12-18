use chrono::{DateTime, Utc};
use rusqlite::{
    types::{FromSql, FromSqlError, ValueRef},
    Row, ToSql,
};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::application::utils;

#[repr(u8)]
#[derive(Clone, Copy, Serialize_repr, Deserialize_repr, PartialEq)]
pub enum PunishmentType {
    None = 0,
    Ban = 1,
    Kick = 2,
    Mute = 3,
}

impl PunishmentType {
    pub(crate) fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl ToSql for PunishmentType {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::from(self.as_u8()))
    }
}

impl FromSql for PunishmentType {
    fn column_result(value: ValueRef<'_>) -> rusqlite::Result<PunishmentType, FromSqlError> {
        let int_value = value.as_i64()?;
        match int_value {
            0 => Ok(PunishmentType::None),
            1 => Ok(PunishmentType::Ban),
            2 => Ok(PunishmentType::Kick),
            3 => Ok(PunishmentType::Mute),
            _ => Err(FromSqlError::OutOfRange(int_value)),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DatabasePunishment {
    pub punishment_id: String,
    pub player_id: u64,
    pub username: String,
    pub steam_id: String,
    pub ip: String,
    pub reason: String,
    pub punishment_duration: u32,
    pub punishment_created_at: DateTime<Utc>,
    pub issuer_steam_id: String,
    pub issuer_name: String,
    pub issuer_ip: String,
    pub punishment_type: PunishmentType,
}

impl DatabasePunishment {
    pub(crate) fn from_row(row: &Row) -> rusqlite::Result<DatabasePunishment> {
        let punishment_date = match utils::time::parse_rfc3339_to_utc(row.get::<_, String>(7)?) {
            Ok(val) => val,
            Err(_) => Utc::now(),
        };

        Ok(DatabasePunishment {
            punishment_id: row.get(0)?,
            player_id: row.get(1)?,
            username: row.get(2)?,
            steam_id: row.get(3)?,
            ip: row.get(4)?,
            reason: row.get(5)?,
            punishment_duration: row.get(6)?,
            punishment_created_at: punishment_date,
            issuer_steam_id: row.get(8)?,
            issuer_name: row.get(9)?,
            issuer_ip: row.get(10)?,
            punishment_type: row.get(11)?,
        })
    }
}
