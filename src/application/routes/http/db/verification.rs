use rocket::{http::Status, serde::json::Json};
use std::sync::Arc;

use crate::application::{
    application::Application,
    db::player::structs::{
        DatabaseModifyPlayerVerification, DatabasePlayerVerification, PlayerVerification,
    },
    routes::http::DbAuthHeader,
};
use rocket::State;
use tokio::sync::Mutex;

#[get("/db/player/verification/get/id/<player_id>")]
// MARK: Get player verification by player_id
pub async fn db_get_player_verification_by_player_id(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    player_id: u64,
) -> Result<Json<DatabasePlayerVerification>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            db_handler
                .player_database
                .get_player_verification(player_id)
                .map_err(|_| Status::InternalServerError)
                .map(|val| Json(val))
        }
        None => Err(Status::FailedDependency),
    }
}

#[get("/db/player/verification/get/steam/<steam_id>")]
// MARK: Get player verification by steam_id
pub async fn db_get_player_verification_by_steam_id(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    steam_id: String,
) -> Result<Json<DatabasePlayerVerification>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            let players = db_handler
                .player_database
                .get_players_by_steam(&steam_id)
                .map_err(|_| Status::InternalServerError)?;

            if players.is_empty() {
                return Err(Status::NotFound);
            }

            let verification = DatabasePlayerVerification::from(players[0].clone());
            Ok(Json(verification))
        }
        None => Err(Status::FailedDependency),
    }
}

#[get("/db/player/verification/get/discord/<discord_id>")]
// MARK: Get player verification by discord_id
pub async fn db_get_player_verification_by_discord_id(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    discord_id: String,
) -> Result<Json<DatabasePlayerVerification>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            let players = db_handler
                .player_database
                .get_players_by_discord(&discord_id)
                .map_err(|_| Status::InternalServerError)?;

            if players.is_empty() {
                return Err(Status::NotFound);
            }

            let verification = DatabasePlayerVerification::from(players[0].clone());
            Ok(Json(verification))
        }
        None => Err(Status::FailedDependency),
    }
}

#[get("/db/player/verification/get/code/<code>")]
// MARK: Get player verification by code
pub async fn db_get_player_verification_by_code(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    code: String,
) -> Result<Json<DatabasePlayerVerification>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            let players = db_handler
                .player_database
                .get_all_players()
                .map_err(|_| Status::InternalServerError)?;

            if players.is_empty() {
                return Err(Status::NotFound);
            }

            let output = players
                .iter()
                .filter_map(|player| {
                    player
                        .verification_key
                        .as_ref()
                        .filter(|verify_code| verify_code == &&code)
                        .map(|_| DatabasePlayerVerification::from(player.clone()))
                })
                .next();

            match output {
                Some(data) => Ok(Json(data)),
                None => Err(Status::NotFound),
            }
        }
        None => Err(Status::FailedDependency),
    }
}

#[post(
    "/db/player/verification/add",
    format = "application/json",
    data = "<parsed_data>"
)]
// MARK: Add player verification
pub async fn db_add_player_verification(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    parsed_data: Json<DatabaseModifyPlayerVerification>,
) -> Result<(), Status> {
    let _verification_key = match &parsed_data.0.code {
        Some(val) => val,
        None => return Err(Status::BadRequest),
    };

    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let mut db_handler = db_handler_lock.lock().await;
            db_handler
                .player_database
                .add_player_verification(parsed_data.0)
                .map_err(|_| Status::InternalServerError)
        }
        None => Err(Status::FailedDependency),
    }
}

#[post(
    "/db/player/verification/update",
    format = "application/json",
    data = "<parsed_data>"
)]
// MARK: Verify player
pub async fn db_update_player_verification(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    parsed_data: Json<DatabaseModifyPlayerVerification>,
) -> Result<(), Status> {
    // Check for discord_id in the request
    let discord_id = match &parsed_data.0.discord_id {
        Some(val) => val,
        None => return Err(Status::BadRequest),
    };

    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let mut db_handler = db_handler_lock.lock().await;
            let existing_verified_player = match db_handler
                .player_database
                .get_player_by_discord_or_steam(discord_id, &parsed_data.0.steam_id)
            {
                Ok(Some(player)) => player,
                Ok(None) => return Err(Status::NotFound),
                Err(_) => return Err(Status::InternalServerError),
            };
            if existing_verified_player.is_verified() {
                return Err(Status::AlreadyReported);
            }
            if existing_verified_player.is_verification_banned() {
                return Err(Status::Forbidden);
            }

            // Check if player has already verification, with different account
            let players = db_handler
                .player_database
                .get_players_by_discord(discord_id)
                .map_err(|_| Status::NotModified)?;

            if players.len() > 0 {
                return Err(Status::Forbidden);
            }

            db_handler
                .player_database
                .set_player_verification(
                    existing_verified_player.player_id,
                    PlayerVerification::Success,
                    parsed_data.0.discord_id,
                    None,
                )
                .map_err(|_| Status::BadRequest)
        }
        None => Err(Status::FailedDependency),
    }
}

#[post(
    "/db/player/verification/set",
    format = "application/json",
    data = "<parsed_data>"
)]
// MARK: Set player verification
pub async fn dn_modify_player_verification(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    parsed_data: Json<DatabaseModifyPlayerVerification>,
) -> Result<(), Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let mut db_handler = db_handler_lock.lock().await;
            db_handler
                .player_database
                .set_player_verification(
                    parsed_data.0.player_id,
                    parsed_data.0.verified_status,
                    parsed_data.0.discord_id,
                    parsed_data.0.code,
                )
                .map_err(|_| Status::InternalServerError)
        }
        None => Err(Status::FailedDependency),
    }
}
