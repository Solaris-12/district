use rocket::serde::json::Json;
use serde_json::{json, Value};

pub fn http_response_message_200() -> Json<Value> {
  Json(json!({
    "status": 200,
    "message": "OK"
  }))
}

// pub fn http_response_message_400() -> Json<Value> {
//   Json(json!({
//     "status": 400,
//     "message": "Bad Request"
//   }))
// }

// pub fn http_response_message_404() -> Json<Value> {
//     Json(json!({
//       "status": 404,
//       "message": "Not Found"
//     }))
// }

pub fn http_response_message_500(error: Option<String>) -> Json<Value> {
  Json(json!({
    "status": 500,
    "message": "Internal Server Error",
    "error": error
  }))
}
