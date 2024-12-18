use std::sync::Arc;

use crate::application::application::Application;
use crate::application::db::punishments::structs::DatabasePunishment;
use crate::application::routes::http::DbAuthHeader;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use tokio::sync::Mutex;

#[get("/db/punishments")]
// MARK: Get all punishments
pub async fn db_get_all_punishments(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
) -> Result<Json<Vec<DatabasePunishment>>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            db_handler
                .punishment_database
                .get_all_punishments()
                .map_err(|_| Status::InternalServerError)
                .map(|val| Json(val))
        }
        None => Err(Status::FailedDependency),
    }
}

#[get("/db/punishments/get/<punishment_id>")]
// MARK: Get punishment by it's ID
pub async fn db_get_punishment_by_punishment_id(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    punishment_id: String,
) -> Result<Json<DatabasePunishment>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            db_handler
                .punishment_database
                .get_punishment_by_punishment_id(punishment_id)
                .map_err(|_| Status::InternalServerError)
                .map(|val| Json(val))
        }
        None => Err(Status::FailedDependency),
    }
}

#[get("/db/punishments/get/id/<player_id>")]
// MARK: Get punishment by player_id
pub async fn get_punishments_by_player_id(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    player_id: u64,
) -> Result<Json<Vec<DatabasePunishment>>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            db_handler
                .punishment_database
                .get_punishments_by_player_id(player_id)
                .map_err(|_| Status::InternalServerError)
                .map(|val| Json(val))
        }
        None => Err(Status::FailedDependency),
    }
}

#[get("/db/punishments/get/steam/<steam_id>")]
// MARK: Get punishment by steam_id
pub async fn get_punishments_by_steam_id(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    steam_id: String,
) -> Result<Json<Vec<DatabasePunishment>>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            db_handler
                .punishment_database
                .get_punishments_by_steam_id(steam_id)
                .map_err(|_| Status::InternalServerError)
                .map(|val| Json(val))
        }
        None => Err(Status::FailedDependency),
    }
}

#[get("/db/punishments/get/ip/<ip>")]
// MARK: Get punishment by ip
pub async fn get_punishments_by_ip(
    _auth_header: DbAuthHeader,
    app_data: &State<Arc<Mutex<Application>>>,
    ip: String,
) -> Result<Json<Vec<DatabasePunishment>>, Status> {
    let app = app_data.lock().await;
    match app.databases.as_ref() {
        Some(db_handler_lock) => {
            let db_handler = db_handler_lock.lock().await;
            db_handler
                .punishment_database
                .get_punishments_by_ip(ip)
                .map_err(|_| Status::InternalServerError)
                .map(|val| Json(val))
        }
        None => Err(Status::FailedDependency),
    }
}
