use std::sync::Arc;

use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::application::application::Application;
use crate::application::db::player::structs::{
    DatabasePlayer, DatabasePlayerCount, DatabasePlayerJoin,
};
use crate::application::db::punishments::structs::{DatabasePunishment, PunishmentType};
use crate::application::routes::http::DbAuthHeader;

#[post(
    "/db/player/join/<_server_id>",
    format = "application/json",
    data = "<parsed_data>"
)]
// MARK: On player join
pub async fn db_on_player_join(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    _server_id: u64, // unused for now
    parsed_data: Json<DatabasePlayerJoin>,
) -> Result<Json<DatabasePlayer>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let mut db_handler = db_handler_lock.lock().await;
            db_handler
                .player_database
                .player_joined(parsed_data.0)
                .map_err(|_| Status::InternalServerError)
                .map(|val| Json(val))
        }
        None => Err(Status::FailedDependency),
    }
}

#[get("/db/player/get/all")]
// MARK: Get all players
pub async fn db_get_all_players(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
) -> Result<Json<Vec<DatabasePlayer>>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            db_handler
                .player_database
                .get_all_players()
                .map_err(|_| Status::InternalServerError)
                .map(|val| Json(val))
        }
        None => Err(Status::FailedDependency),
    }
}

#[get("/db/player/get/id/<player_id>")]
// MARK: Get player by player id
pub async fn db_get_player_by_player_id(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    player_id: u64,
) -> Result<Json<DatabasePlayer>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            db_handler
                .player_database
                .get_player_by_id(player_id)
                .map_err(|_| Status::InternalServerError)
                .map(|val| Json(val))
        }
        None => Err(Status::FailedDependency),
    }
}

#[get("/db/player/get/steam/<steam_id>")]
// MARK: Get player by steam id
pub async fn db_get_player_by_steam_id(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    steam_id: &str,
) -> Result<Json<Vec<DatabasePlayer>>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            db_handler
                .player_database
                .get_players_by_steam(steam_id)
                .map_err(|_| Status::InternalServerError)
                .map(|val| Json(val))
        }
        None => Err(Status::FailedDependency),
    }
}

#[get("/db/player/get/discord/<discord_id>")]
// MARK: Get player by discord id
pub async fn db_get_player_by_discord_id(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    discord_id: &str,
) -> Result<Json<Vec<DatabasePlayer>>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            db_handler
                .player_database
                .get_players_by_discord(discord_id)
                .map_err(|_| Status::InternalServerError)
                .map(|val| Json(val))
        }
        None => Err(Status::FailedDependency),
    }
}

#[derive(Serialize, Deserialize)]
pub struct DatabasePlayerPunishment {
    username: String,
    steam_id: String,
    ip: String,
    reason: String,
    punishment_duration: u32,
    issuer_steam_id: String,
    issuer_name: String,
    issuer_ip: String,
    punishment_type: PunishmentType,
}

#[post(
    "/db/player/punishment/add/<player_id>",
    format = "application/json",
    data = "<parsed_data>"
)]
// MARK: Add punishment to player
pub async fn db_add_punishment_to_player(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    player_id: u64,
    parsed_data: Json<DatabasePlayerPunishment>,
) -> Result<Json<DatabasePunishment>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let mut db_handler = db_handler_lock.lock().await;
            let mut player = db_handler
                .player_database
                .get_player_by_id(player_id)
                .map_err(|_| Status::ExpectationFailed)?;

            let punishment_id: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(8)
                .map(char::from)
                .collect();

            let data = DatabasePunishment {
                punishment_id: (&punishment_id).to_string(),
                player_id,
                username: parsed_data.0.username,
                steam_id: parsed_data.0.steam_id,
                ip: parsed_data.0.ip,
                reason: parsed_data.0.reason,
                punishment_duration: parsed_data.0.punishment_duration,
                punishment_created_at: Utc::now(),
                issuer_steam_id: parsed_data.0.issuer_steam_id,
                issuer_name: parsed_data.0.issuer_name,
                issuer_ip: parsed_data.0.issuer_ip,
                punishment_type: parsed_data.0.punishment_type,
            };

            match db_handler.punishment_database.create_new_punishment(&data) {
                Ok(_) => {
                    if let Some(ban_ids) = &mut player.ban_ids {
                        ban_ids.push(punishment_id);
                    } else {
                        player.ban_ids = Some(vec![punishment_id]);
                    }
                    match db_handler.player_database.modify_player(player_id, player) {
                        Ok(_) => Ok(Json(data)),
                        Err(_) => Err(Status::InternalServerError),
                    }
                }
                Err(_) => Err(Status::NotModified),
            }
        }
        None => Err(Status::FailedDependency),
    }
}

#[get("/db/player/count?<since>")]
// MARK: Get player count
pub async fn db_get_player_count(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    since: Option<u64>,
) -> Result<Json<Vec<DatabasePlayerCount>>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            if let Some(timestamp) = since {
                db_handler
                    .player_database
                    .get_player_count_from(timestamp)
                    .map_err(|_| Status::InternalServerError)
                    .map(|val| Json(val))
            } else {
                db_handler
                    .player_database
                    .get_player_count()
                    .map_err(|_| Status::InternalServerError)
                    .map(|val| Json(val))
            }
        }
        None => Err(Status::FailedDependency),
    }
}

#[post(
    "/db/player/count",
    format = "application/json",
    data = "<parsed_data>"
)]
// MARK: Set player count
pub async fn db_set_some_player_count(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    parsed_data: Json<DatabasePlayerCount>,
) -> Result<(), Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            db_handler
                .player_database
                .set_player_count(parsed_data.0)
                .map_err(|_| Status::InternalServerError)
        }
        None => Err(Status::FailedDependency),
    }
}
