use chrono::DateTime;
use rocket::{http::Status, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::application::{
  application::Application,
  db::leaderboards::structs::{LeaderboardRecord, LeaderboardRecordType},
  routes::{http::DbAuthHeader, messages::http_response_message_200},
  utils,
};

#[get("/db/leaderboards/get?<player_id>&<type>")]
// MARK: Get player leaderboards by player_id and type
pub async fn db_get_player_leaderboards(
  _auth_header: DbAuthHeader,
  app_data: &State<Arc<Mutex<Application>>>,
  player_id: Option<u64>,
  r#type: Option<u8>,
) -> Result<Json<Vec<LeaderboardRecord>>, Status> {
  let app = app_data.lock().await;
  let kind = r#type.and_then(|val| LeaderboardRecordType::from_u8(val).ok());

  match app.databases.as_ref() {
    Some(db_handler_lock) => {
      let db_handler = db_handler_lock.lock().await;

      match db_handler.leaderboard_database.as_ref() {
        Some(leaderboard_db) => {
          if let (Some(p_id), Some(kind)) = (player_id, kind) {
            leaderboard_db
              .get_all_from_player_by_type(p_id, kind)
              .map_err(|_| Status::InternalServerError)
              .map(|val| Json(val))
          } else if let Some(p_id) = player_id {
            leaderboard_db
              .get_all_from_player(p_id)
              .map_err(|_| Status::InternalServerError)
              .map(|val| Json(val))
          } else if let Some(kind) = kind {
            leaderboard_db
              .get_all_by_type(kind)
              .map_err(|_| Status::InternalServerError)
              .map(|val| Json(val))
          } else {
            leaderboard_db
              .get_all_data()
              .map_err(|_| Status::InternalServerError)
              .map(|val| Json(val))
          }
        }
        None => Err(Status::NoContent),
      }
    }
    None => Err(Status::FailedDependency),
  }
}

#[patch("/db/leaderboards/clear?<player_id>&<type>")]
// MARK: Clear leaderboards
pub async fn db_clear_leaderboards(
  _auth_header: DbAuthHeader,
  app_data: &State<Arc<Mutex<Application>>>,
  player_id: u64,
  r#type: Option<u8>,
) -> Result<Json<Value>, Status> {
  let app = app_data.lock().await;
  let kind = r#type.and_then(|val| LeaderboardRecordType::from_u8(val).ok());

  match app.databases.as_ref() {
    Some(db_handler_lock) => {
      let db_handler = db_handler_lock.lock().await;

      match db_handler.leaderboard_database.as_ref() {
        Some(leaderboard_db) => {
          if let Some(kind) = kind {
            leaderboard_db
              .clear_all_from_player_by_type(player_id, kind)
              .map_err(|_| Status::InternalServerError)
              .map(|_| http_response_message_200())
          } else {
            leaderboard_db
              .clear_all_from_player(player_id)
              .map_err(|_| Status::InternalServerError)
              .map(|_| http_response_message_200())
          }
        }
        None => Err(Status::NoContent),
      }
    }
    None => Err(Status::FailedDependency),
  }
}

#[patch("/db/leaderboards/remove?<player_id>&<timestamp>")]
// MARK: Remove leaderboard record by date
pub async fn db_leaderboards_remove_by_date(
  _auth_header: DbAuthHeader,
  app_data: &State<Arc<Mutex<Application>>>,
  player_id: u64,
  timestamp: u64,
) -> Result<Json<Value>, Status> {
  let app = app_data.lock().await;

  match app.databases.as_ref() {
    Some(db_handler_lock) => {
      let db_handler = db_handler_lock.lock().await;

      match db_handler.leaderboard_database.as_ref() {
        Some(leaderboard_db) => leaderboard_db
          .remove_from_player_by_date(
            player_id,
            DateTime::from_timestamp(timestamp as i64, 0).unwrap(),
          )
          .map(|_| http_response_message_200())
          .map_err(|_| Status::InternalServerError),
        None => Err(Status::NoContent),
      }
    }
    None => Err(Status::FailedDependency),
  }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct LeaderboardStatRequest {
  pub player_id: u64,
  #[serde(rename = "type")]
  pub r#type: LeaderboardRecordType,
  pub value: f64,
  pub date_time: u64,
}

#[post(
  "/db/leaderboards/add",
  format = "application/json",
  data = "<parsed_data>"
)]
// MARK: Add stat to player
pub async fn db_add_stat_to_players_leaderboards(
  _auth_header: DbAuthHeader,
  app_data: &State<Arc<Mutex<Application>>>,
  parsed_data: Json<LeaderboardStatRequest>,
) -> Result<Json<Value>, Status> {
  let app = app_data.lock().await;
  match app.databases.as_ref() {
    Some(db_handler_lock) => {
      let db_handler = db_handler_lock.lock().await;
      let date_time =
        utils::time::parse_rfc3339_to_utc(parsed_data.0.date_time.to_string()).unwrap();

      match db_handler.leaderboard_database.as_ref() {
        Some(leaderboard_db) => leaderboard_db
          .add_stat_to_player(
            parsed_data.0.player_id,
            parsed_data.0.r#type,
            parsed_data.0.value,
            date_time,
          )
          .map(|_| http_response_message_200())
          .map_err(|_| Status::InternalServerError),
        None => Err(Status::NoContent),
      }
    }
    None => Err(Status::FailedDependency),
  }
}
