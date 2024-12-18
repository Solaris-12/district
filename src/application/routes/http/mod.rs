use std::sync::Arc;

use rocket::{
    http::Status,
    request::{self, FromRequest},
    Request, State,
};
use tokio::sync::Mutex;

use crate::application::{application::Application, config::config::ConfigApp};

pub(crate) mod db;
pub(crate) mod log_routes;
pub(crate) mod r#static;

pub struct DbAuthHeader(String);
pub struct LogAuthHeader(String);

#[derive(Debug)]
pub enum AuthError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for LogAuthHeader {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // Access the managed state
        let managed_state = request.guard::<&State<Arc<Mutex<Application>>>>().await;

        // Check if the managed state is available
        match managed_state {
            request::Outcome::Success(state) => {
                let app = state.lock().await;

                match request.headers().get_one("Authorization") {
                    Some(auth_header) => {
                        if check_log_auth(auth_header.to_owned(), app.config.as_ref().unwrap()) {
                            request::Outcome::Success(LogAuthHeader(auth_header.to_string()))
                        } else {
                            request::Outcome::Error((Status::Unauthorized, AuthError::Invalid))
                        }
                    }
                    None => request::Outcome::Error((Status::Unauthorized, AuthError::Missing)),
                }
            }
            request::Outcome::Error(_) | request::Outcome::Forward(_) => {
                // Handle the case where the managed state is not available
                request::Outcome::Error((Status::InternalServerError, AuthError::Missing))
            }
        }
    }
}

pub fn check_log_auth(token: String, config: &ConfigApp) -> bool {
    let log_token = &config.auth.log;
    if token == *log_token {
        return true;
    }
    return false;
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DbAuthHeader {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // Access the managed state
        let managed_state = request.guard::<&State<Arc<Mutex<Application>>>>().await;

        // Check if the managed state is available
        match managed_state {
            request::Outcome::Success(state) => {
                let app = state.lock().await;

                match request.headers().get_one("Authorization") {
                    Some(auth_header) => {
                        if check_db_auth(auth_header.to_owned(), app.config.as_ref().unwrap()) {
                            request::Outcome::Success(DbAuthHeader(auth_header.to_string()))
                        } else {
                            request::Outcome::Error((Status::Unauthorized, AuthError::Invalid))
                        }
                    }
                    None => request::Outcome::Error((Status::Unauthorized, AuthError::Missing)),
                }
            }
            request::Outcome::Error(_) | request::Outcome::Forward(_) => {
                // Handle the case where the managed state is not available
                request::Outcome::Error((Status::InternalServerError, AuthError::Missing))
            }
        }
    }
}

pub fn check_db_auth(token: String, config: &ConfigApp) -> bool {
    let db_token = &config.auth.db;
    if token == *db_token {
        return true;
    }
    return false;
}
