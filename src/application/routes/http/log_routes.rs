use std::{collections::HashMap, sync::Arc};

use rocket::{serde::json::Json, State};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::application::{
  self,
  application::Application,
  features::logs::JsonValueNotObject,
  routes::messages::{http_response_message_200, http_response_message_500},
  server::status::DistrictServerStatus,
};

use super::LogAuthHeader;

/// # Handles POST requests to set Server status (players online, tps, etc...)
///
/// ## Arguments
///
/// * `status_data` - JSON data containing [`crate::application::server::status::DistrictServerStatus`] information.
#[post(
  "/server/<server_id>/status",
  format = "application/json",
  data = "<status_data>"
)]
pub async fn log_server_status(
  _auth_header: LogAuthHeader,
  app_data: &State<Arc<Mutex<Application>>>,
  server_id: u64,
  status_data: Json<DistrictServerStatus>,
) -> Json<Value> {
  let app = app_data.lock().await;
  if let Some(db_lock) = &app.databases {
    let mut db_handler = db_lock.lock().await;
    if let Err(player_count_error) = db_handler
      .player_database
      .set_player_count_auto(status_data.0.player_count.clone().into())
    {
      return http_response_message_500(Some(player_count_error.to_string()));
    }
    if let Some(player_list) = &status_data.0.player_ids {
      for player in player_list.iter() {
        let _ = db_handler.player_database.add_playtime_to_player(
          player.clone(),
          status_data.0.duration_since_last.unwrap_or(0.1f32),
        );
      }
    }
  }
  match app.try_get_server(server_id).await {
    Ok(server_lock) => {
      let mut server = server_lock.lock().await;
      server.status = status_data.0;
      http_response_message_200()
    }
    Err(e) => http_response_message_500(Some(e.to_string())),
  }
}

#[post(
  "/logs/<server_id>/<translation>",
  format = "application/json",
  data = "<parsed_data>"
)]
pub async fn log_with_translation(
  _auth_header: LogAuthHeader,
  app_data: &State<Arc<Mutex<Application>>>,
  server_id: u64,
  translation: &str,
  parsed_data: Json<HashMap<String, JsonValueNotObject>>,
) -> Json<Value> {
  match application::features::logs::handle_log_with_data(
    app_data.inner(),
    server_id,
    translation,
    parsed_data.0,
  )
  .await
  {
    Ok(_) => http_response_message_200(),
    Err(e) => http_response_message_500(Some(e)),
  }
}
