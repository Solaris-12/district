use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::application::{self, application::Application, features::logs::JsonValueNotObject};

#[derive(Deserialize)]
pub(super) struct WsLogData {
    server_id: u64,
    translation: String,
    data: HashMap<String, JsonValueNotObject>,
}

pub async fn ws_log_with_translation(
    app_data: &Arc<Mutex<Application>>,
    data: &Value,
) -> Result<String, String> {
    let data_parsed: WsLogData = serde_json::from_value(data.clone()).map_err(|e| e.to_string())?;

    match application::features::logs::handle_log_with_data(
        app_data,
        data_parsed.server_id,
        &data_parsed.translation,
        data_parsed.data,
    )
    .await
    {
        Ok(_) => Ok(String::from("Success")),
        Err(e) => Err(e),
    }
}
