use std::error::Error;

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
// MARK: (enum) Leaderboard Record Type
pub(crate) enum LeaderboardRecordType {
  PlayTime = 0,
  Kills = 1,
  Deaths = 2,
  Wins = 3,
  Losses = 4,
  Assists = 5,
}

impl LeaderboardRecordType {
  fn as_u8(&self) -> u8 {
    *self as u8
  }

  pub fn from_u8(val: u8) -> Result<Self, Box<dyn Error>> {
    match val {
      0 => Ok(LeaderboardRecordType::PlayTime),
      1 => Ok(LeaderboardRecordType::Kills),
      2 => Ok(LeaderboardRecordType::Deaths),
      3 => Ok(LeaderboardRecordType::Wins),
      4 => Ok(LeaderboardRecordType::Losses),
      5 => Ok(LeaderboardRecordType::Assists),
      _ => panic!("Invalid Leaderboard Record Type"),
    }
  }
}

impl ToSql for LeaderboardRecordType {
  fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>> {
    Ok(rusqlite::types::ToSqlOutput::from(self.as_u8()))
  }
}

impl FromSql for LeaderboardRecordType {
  fn column_result(value: ValueRef<'_>) -> Result<LeaderboardRecordType, FromSqlError> {
    let int_value = value.as_i64()?;
    let unsigned_int_value: u8 = int_value
      .try_into()
      .map_err(|_| FromSqlError::OutOfRange(int_value))?;
    LeaderboardRecordType::from_u8(unsigned_int_value)
      .map_err(|_| FromSqlError::OutOfRange(int_value))
  }
}

#[derive(Serialize, Deserialize)]
// MARK: (struct) Leaderboard Record
pub(crate) struct LeaderboardRecord {
  pub player_id: u64,
  #[serde(rename = "type")]
  pub r#type: LeaderboardRecordType,
  pub value: f64,
  pub date_time: DateTime<Utc>,
}

impl LeaderboardRecord {
  pub fn from_row(row: &rusqlite::Row) -> Result<Self> {
    let date_time = match utils::time::parse_rfc3339_to_utc(row.get::<_, String>(3)?) {
      Ok(val) => val,
      Err(_) => Utc::now(),
    };

    Ok(Self {
      player_id: row.get(0)?,
      r#type: row.get(1)?,
      value: row.get(2)?,
      date_time,
    })
  }
}
