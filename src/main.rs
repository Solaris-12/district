pub mod application;
pub mod logger;

#[macro_use]
extern crate rocket;

use rocket::{figment::Figment, Route};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::logger::{LogLevel, Logger};
use application::application::Application;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, EnvFilter};

async fn setup_app() -> Arc<Mutex<Application>> {
  let app = Application::new();
  let out = app
    .setup(std::env::var("DISTRICT_CONFIG").unwrap_or("./config.json".to_string()))
    .await;
  out
}

#[launch]
async fn rocket() -> _ {
  let app_lock = setup_app().await;
  let routes: Vec<Route>;
  let figment: Figment;

  {
    let app = app_lock.lock().await;

    let filter = EnvFilter::builder()
      .with_default_directive(LevelFilter::INFO.into())
      .with_env_var("RUST_LOG")
      .from_env_lossy();

    tracing_subscriber::fmt()
      .with_env_filter(filter)
      .fmt_fields(fmt::format::DefaultFields::new())
      .event_format(
        fmt::format::Format::default()
          .without_time()
          .with_target(false)
          .with_thread_names(false)
          .with_thread_ids(false)
          .with_file(false)
          .with_source_location(false),
      )
      .init();

    log_i!("Starting rocket server");
    routes = match app.router.as_ref() {
      Some(router) => router.get_routes().to_vec(),
      None => {
        log_e!("Router not found");
        panic!("Router not found");
      }
    };

    let app_config = app.config.as_ref().unwrap();
    figment = rocket::Config::figment()
      .merge(("port", app_config.server_port))
      .merge(("address", app_config.server_address.clone()));
  }

  rocket::custom(figment)
    .mount("/", routes)
    .manage(app_lock)
    .register("/", catchers![/* To be added (TODO: HTTP catchers)*/])
}
