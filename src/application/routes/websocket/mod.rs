pub(super) mod logs;
pub(super) mod stats;
pub(crate) mod structs;

use rocket_ws::{Channel, WebSocket};
use std::{
  collections::VecDeque,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  time::Duration,
};

use rocket::{
  futures::{SinkExt as _, StreamExt as _},
  http::Status,
  request::{self, FromRequest},
  Request, State,
};
use tokio::{sync::Mutex, time::interval};

use crate::{
  application::{
    application::Application,
    config::config::ConfigApp,
    routes::websocket::structs::{
      WebsocketIncomingMessage, WebsocketRoute, WsMessageResponse, WsReponseStatus,
      WsResponseCreate as _, WsResponses,
    },
  },
  log_d, log_i, log_w, log_x,
  logger::{LogLevel, Logger},
};

pub struct WebsocketAuthHeader(String);

#[derive(Debug)]
pub enum WebsocketAuthError {
  Missing,
  Invalid,
}

// MARK: Authorization
#[rocket::async_trait]
impl<'r> FromRequest<'r> for WebsocketAuthHeader {
  type Error = WebsocketAuthError;

  async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
    // Access the managed state
    let managed_state = request.guard::<&State<Arc<Mutex<Application>>>>().await;

    // Check if the managed state is available
    match managed_state {
      request::Outcome::Success(state) => {
        let app = state.lock().await;

        match request.cookies().get("authorization") {
          Some(auth_cookie) => {
            let auth_token = auth_cookie.value().to_string();
            if check_ws_auth(auth_token.clone(), app.config.as_ref().unwrap()) {
              return request::Outcome::Success(WebsocketAuthHeader(auth_token));
            } else {
              return request::Outcome::Error((Status::Unauthorized, WebsocketAuthError::Invalid));
            }
          }
          None => {
            if let Some(auth_header) = request.headers().get_one("Authorization") {
              if check_ws_auth(auth_header.to_string(), app.config.as_ref().unwrap()) {
                return request::Outcome::Success(WebsocketAuthHeader(auth_header.to_string()));
              } else {
                return request::Outcome::Error((
                  Status::Unauthorized,
                  WebsocketAuthError::Invalid,
                ));
              }
            } else {
              return request::Outcome::Error((Status::Unauthorized, WebsocketAuthError::Missing));
            }
          }
        }
      }
      request::Outcome::Error(_) | request::Outcome::Forward(_) => {
        // Handle the case where the managed state is not available
        request::Outcome::Error((Status::InternalServerError, WebsocketAuthError::Missing))
      }
    }
  }
}

pub fn check_ws_auth(token: String, config: &ConfigApp) -> bool {
  let log_token = &config.auth.ws;
  if token == *log_token {
    return true;
  }
  return false;
}

// MARK: Websocket connection
#[get("/ws/<server_id>/connect")]
pub fn websocket_connect(
  _auth_header: WebsocketAuthHeader,
  ws: WebSocket,
  server_id: u64,
  app_data: &State<Arc<Mutex<Application>>>,
) -> Channel<'static> {
  let app_arc = Arc::clone(app_data.inner());
  let app_messenger_arc = Arc::clone(app_data.inner());
  log_i!(format!(
    "NEW WebSocket Connection! Server '{}' has connected!",
    &server_id
  ));

  ws.channel(move |stream| {
    Box::pin(async move {
      let (mut sink, mut stream) = stream.split();
      let mut interval = interval(Duration::from_secs(1));
      let messages_to_send: Arc<Mutex<VecDeque<WsResponses>>> =
        Arc::new(Mutex::new(VecDeque::new()));
      let messages_to_send_clone = Arc::clone(&messages_to_send);
      let connected = Arc::new(AtomicBool::new(true));
      let connected_clone = Arc::clone(&connected);

      // MARK: - - Response handler
      tokio::spawn(async move {
        loop {
          interval.tick().await;
          if !connected.load(Ordering::Relaxed) {
            log_d!(format!(
              "WebSocket response handler disconnected {}",
              server_id
            ));
            return;
          }

          {
            let app_lock =
              match tokio::time::timeout(Duration::from_secs(1), app_messenger_arc.lock()).await {
                Ok(app_lock) => app_lock,
                Err(_) => {
                  log_d!(format!("WebSocket message handler timeout {}", server_id));
                  break;
                }
              };
            let server_lock = match tokio::time::timeout(
              Duration::from_secs(1),
              app_lock.try_get_server(server_id),
            )
            .await
            {
              Ok(server_lock) => match server_lock {
                Ok(server_lock) => server_lock,
                Err(_) => {
                  log_d!(format!(
                    "WebSocket message handler could not get server {}",
                    server_id
                  ));
                  break;
                }
              },
              Err(_) => {
                log_d!(format!(
                  "WebSocket message handler server lock timeout {}",
                  server_id
                ));
                break;
              }
            };
            let mut server =
              match tokio::time::timeout(Duration::from_secs(1), server_lock.lock()).await {
                Ok(server) => server,
                Err(_) => {
                  log_d!(format!(
                    "WebSocket message handler could not lock server {}",
                    server_id
                  ));
                  break;
                }
              };
            for msg in server.ws_msgs.iter() {
              log_x!(
                LogLevel::Dev,
                format!("Sending message: {}", serde_json::to_string(&msg).unwrap())
              );
              if let Err(err) = sink
                .send(rocket_ws::Message::Text(
                  serde_json::to_string(&msg).unwrap(),
                ))
                .await
              {
                log_w!(format!("Error sending message: {}", err));
              }
            }
            server.ws_msgs.clear();
          }

          {
            let mut msgs = messages_to_send.lock().await;
            let msgs_len = msgs.len();
            for msg in msgs.drain(0..msgs_len) {
              let _ = sink
                .send(rocket_ws::Message::Text(
                  serde_json::to_string(&msg).unwrap(),
                ))
                .await;
            }
          }
        }
      });

      // MARK: - - Incoming parser
      while let Some(message) = stream.next().await {
        match message {
          Ok(incoming_msg) => {
            // MARK: - - - Text message
            if incoming_msg.is_text() {
              match serde_json::from_str::<WebsocketIncomingMessage>(
                &incoming_msg.into_text().unwrap(),
              ) {
                Ok(route_message) => {
                  let response = match route_message.route {
                    WebsocketRoute::LogsRoute => {
                      logs::ws_log_with_translation(&app_arc, &route_message.data).await
                    }
                    WebsocketRoute::StatsRoute => {
                      stats::ws_stats_add_to_player(&app_arc, &route_message.data).await
                    } // ...
                  };

                  let response_text = match response {
                    Ok(data) => WsMessageResponse::create(WsReponseStatus::Ok, "Ok", Some(&data)),
                    Err(e) => {
                      WsMessageResponse::create(WsReponseStatus::Error, "Internal Error", Some(&e))
                    }
                  };

                  {
                    let mut msgs = messages_to_send_clone.lock().await;
                    msgs.push_back(response_text);
                  }
                }
                Err(e) => {
                  log_d!(format!("Websocket received bad request! : {}", e));
                  {
                    let mut msgs = messages_to_send_clone.lock().await;
                    msgs.push_back(WsMessageResponse::create(
                      WsReponseStatus::BadRequest,
                      "Bad Request",
                      Some(&e),
                    ));
                  }
                }
              }
            }
          }
          Err(e) => {
            log_w!(format!("Websocket connection disconnected: {}", e));
            connected_clone.store(false, Ordering::Relaxed);
          }
        }
      }
      Ok(())
    })
  })
}
