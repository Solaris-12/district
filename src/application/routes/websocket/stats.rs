use std::sync::Arc;

use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::application::{
  application::Application, db::leaderboards::structs::LeaderboardRecordType,
};

#[derive(Deserialize)]
pub(super) struct WsLeaderboardData {
  player_id: u64,
  #[serde(rename = "type")]
  r#type: LeaderboardRecordType,
  value: f64,
}

pub async fn ws_stats_add_to_player(
  app_data: &Arc<Mutex<Application>>,
  data: &Value,
) -> Result<String, String> {
  let app = app_data.lock().await;
  let data_parsed: WsLeaderboardData =
    serde_json::from_value(data.clone()).map_err(|e| e.to_string())?;

  match app.databases.as_ref() {
    Some(db_handler_lock) => {
      let db_handler = db_handler_lock.lock().await;
      match db_handler.leaderboard_database.as_ref() {
        Some(leaderboard_db) => leaderboard_db
          .add_stat_to_player(
            data_parsed.player_id,
            data_parsed.r#type,
            data_parsed.value,
            Utc::now(),
          )
          .map(|_| String::from("Successfully added stat to leaderboards"))
          .map_err(|e| e.to_string()),
        None => Err(String::from("Leaderboards are not enabled on this server")),
      }
    }
    None => Err(String::from("No databases loaded")),
  }
}
