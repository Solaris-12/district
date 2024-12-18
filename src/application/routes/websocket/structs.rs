use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[repr(u8)]
#[derive(Debug, Deserialize_repr, Serialize_repr)]
pub(crate) enum WebsocketRoute {
  LogsRoute = 0,
  StatsRoute = 1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebsocketIncomingMessage {
  pub route: WebsocketRoute,
  pub data: Value,
}

#[repr(u16)]
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)]
pub(crate) enum WsReponseStatus {
  Ok = 200,
  BadRequest = 400,
  WrongAuth = 421,
  Error = 500,
}

#[repr(u16)]
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)]
pub(crate) enum WsResponseType {
  Basic = 0,
  Message = 1,
  Command = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum WsResponses {
  Basic(WsBasicResponse),
  Message(WsMessageResponse),
  Command(WsCommandResponse),
}

pub(crate) trait WsResponseCreate {
  fn create(
    status: WsReponseStatus,
    message: impl ToString,
    data: Option<impl ToString>,
  ) -> WsResponses;
}

// MARK: Basic response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WsBasicResponse {
  #[serde(rename = "type")]
  r#type: WsResponseType,
  status: WsReponseStatus,
  message: String,
}

impl WsResponseCreate for WsBasicResponse {
  fn create(
    status: WsReponseStatus,
    message: impl ToString,
    _data: Option<impl ToString>,
  ) -> WsResponses {
    let response = Self {
      r#type: WsResponseType::Basic,
      status,
      message: message.to_string(),
    };
    WsResponses::Basic(response)
  }
}

// MARK: Message response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WsMessageResponse {
  #[serde(rename = "type")]
  r#type: WsResponseType,
  status: WsReponseStatus,
  message: String,
  response: String,
}

impl WsResponseCreate for WsMessageResponse {
  fn create(
    status: WsReponseStatus,
    message: impl ToString,
    data: Option<impl ToString>,
  ) -> WsResponses {
    let response = Self {
      r#type: WsResponseType::Message,
      status,
      message: message.to_string(),
      response: data.map(|v| v.to_string()).unwrap_or(String::new()),
    };
    WsResponses::Message(response)
  }
}

// MARK: Command response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WsCommandResponse {
  #[serde(rename = "type")]
  r#type: WsResponseType,
  status: WsReponseStatus,
  message: String,
  command: String,
}

impl WsResponseCreate for WsCommandResponse {
  fn create(
    status: WsReponseStatus,
    message: impl ToString,
    data: Option<impl ToString>,
  ) -> WsResponses {
    let response = Self {
      r#type: WsResponseType::Command,
      status,
      message: message.to_string(),
      command: data.map(|v| v.to_string()).unwrap_or(String::new()),
    };
    WsResponses::Command(response)
  }
}
