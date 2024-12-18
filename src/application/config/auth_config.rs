use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigAuth {
  pub db: String,
  pub log: String,
  pub ws: String,
}
