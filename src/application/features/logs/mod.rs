use std::{collections::HashMap, sync::Arc};

use crate::{
  application::application::Application, application::utils::time::get_discord_timestamp,
};
use serde::{Deserialize, Deserializer};
use serde_json::{Number, Value};

use tokio::sync::Mutex;

use super::lang::get_translation;

#[derive(Clone, Eq, PartialEq)]
pub enum JsonValueNotObject {
  Null,
  Bool(bool),
  Number(Number),
  String(String),
  Array(Vec<String>),
}

impl<'de> Deserialize<'de> for JsonValueNotObject {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let value = Value::deserialize(deserializer)?;
    match value {
      Value::Null => Ok(JsonValueNotObject::Null),
      Value::Bool(b) => Ok(JsonValueNotObject::Bool(b)),
      Value::Number(n) => Ok(JsonValueNotObject::Number(n)),
      Value::String(s) => Ok(JsonValueNotObject::String(s)),
      Value::Array(a) => {
        let string_vec: Result<Vec<String>, _> = a
          .iter()
          .map(|v| {
            v.as_str()
              .ok_or(serde::de::Error::custom("Expected string in array"))
              .and_then(|s| Ok(s.to_string()))
          })
          .collect::<Result<Vec<_>, _>>();
        match string_vec {
          Ok(vec) => Ok(JsonValueNotObject::Array(vec)),
          Err(e) => Err(e),
        }
      }
      _ => Err(serde::de::Error::custom("Unsupported JSON value")),
    }
  }
}

// MARK: Log with data
pub async fn handle_log_with_data(
  app: &Arc<Mutex<Application>>,
  server_id: u64,
  translation: &str,
  parsed_data: HashMap<String, JsonValueNotObject>,
) -> Result<(), String> {
  let application = app.lock().await;
  let server_lock = application
    .try_get_server(server_id)
    .await
    .map_err(|e| e.to_string())?;
  let mut server = server_lock.lock().await;

  let key = format!("logs.{}", translation);
  let mut message = match get_translation(application.translations.as_ref().unwrap(), &key) {
    Some(val) => val,
    None => key,
  };

  if message.is_empty() {
    return Ok(());
  }

  message = format!(
    "{}: {}",
    get_discord_timestamp(application.translations.as_ref()),
    message
  );

  for (key, value) in parsed_data.iter() {
    let value_str = match value {
      JsonValueNotObject::String(s) => s.clone(),
      JsonValueNotObject::Number(n) => n.to_string(),
      JsonValueNotObject::Bool(b) => b.to_string(),
      JsonValueNotObject::Null => "null".to_string(),
      JsonValueNotObject::Array(a) => a.join(", "),
    };
    message = message.replace(&format!("{{{}}}", key), &value_str);
  }

  match server.send_message(message).await {
    Ok(_) => Ok(()),
    Err(e) => Err(e.to_string()),
  }
}
