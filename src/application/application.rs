use std::{collections::HashMap, path::PathBuf, sync::Arc};

use rocket::request::{self, FromRequest};
use rocket::Request;
use tokio::sync::Mutex;

use crate::application::features;
use crate::application::timer::timer_loop;
use crate::logger::{LogLevel, Logger};
use crate::{log_d, log_e, log_i, log_x};

use super::{
  bot::bot::DistrictBot, config::config::ConfigApp, db::database_handler::DatabaseHandler,
  routes::DistrictRouter, server::server::DistrictServer,
};

pub const APPLICATION_ICON_URL: &str = "https://imgur.com/xiBYGqs.png";

#[derive(Debug, Clone)]
pub struct Application {
  main_bot: Option<DistrictBot>,
  pub(crate) servers: Vec<Arc<Mutex<DistrictServer>>>,
  pub(crate) config: Option<ConfigApp>,
  config_path: PathBuf,
  pub(crate) translations: Option<HashMap<String, String>>,
  pub(crate) databases: Option<Arc<Mutex<DatabaseHandler>>>,
  pub(crate) router: Option<DistrictRouter>,
}

impl Application {
  pub fn new() -> Self {
    Application {
      main_bot: None,
      servers: vec![],
      config: None,
      config_path: PathBuf::new(),
      translations: None,
      databases: None,
      router: None,
    }
  }

  pub async fn setup(mut self, config_path: String) -> Arc<Mutex<Application>> {
    log_i!("Starting...");
    let cfg_path = PathBuf::from(config_path);
    self.config_path = cfg_path.clone();

    // Setting up config
    let cfg: ConfigApp;
    if cfg_path.exists() {
      log_d!("Config found!");
      cfg = match ConfigApp::load_from_json(cfg_path.clone()) {
        Ok(val) => val,
        Err(err) => {
          panic!("Couldn't read config {:?}: {}", cfg_path, err);
        }
      };
    } else {
      log_d!("Config not found! Creating new one!");
      cfg = ConfigApp::create();
      if let Err(err) = cfg.save_to_json(self.config_path.clone()) {
        log_e!(format!("Error when saving config: {}", err))
      }
    }
    self.config = Some(cfg.clone());

    log_d!("Checking translations!");
    if let Err(e) = features::lang::init_lang(cfg.lang_path.clone().into()) {
      log_e!(format!("Couldn't create new lang file properly: {}", e));
    };

    self.translations = match features::lang::read_lang(cfg.lang_path.clone().into()) {
      Ok(val) => Some(val),
      Err(e) => {
        log_e!(format!("Couldn't load lang file properly: {}", e));
        Some(HashMap::new())
      }
    };

    // Setting up databases
    self.databases = Some(Arc::new(Mutex::new(DatabaseHandler::create(
      cfg.databases.clone(),
    ))));

    // Spawning district guard bot
    log_d!("Booting main bot!");
    let mut main_bot = DistrictBot::new(cfg.main_bot, None, None);
    main_bot.spawn(String::from("DISTRICT_MAIN")).await;
    self.main_bot = Some(main_bot);

    let self_arc = Arc::new(Mutex::new(self));

    log_d!("Checking servers!");
    {
      // Setting up all servers from config
      let mut servers: Vec<Arc<Mutex<DistrictServer>>> = vec![];
      for server in cfg.servers {
        let cloned_arc = Arc::clone(&self_arc);
        let srv = DistrictServer::new(cloned_arc, server.clone()).await;
        log_d!(format!("Added server '{}'", server.name));
        servers.push(srv);
      }
      self_arc.lock().await.servers = servers;
    }
    log_d!("Servers added successfully!");

    {
      let mut app = self_arc.lock().await;

      log_d!("Starting router..");
      app.router = Some(DistrictRouter::new());
    }

    log_d!("Summoning main loop!");

    // Spawning new thread with loop
    let cloned_arc = Arc::clone(&self_arc);
    tokio::spawn(async move {
      match timer_loop(cloned_arc).await {
        Ok(_) => log_i!("Timer loop ended!"),
        Err(e) => log_e!(format!("Timer loop threw an exception: {:?}", e)),
      }
    });

    log_i!("App started successfully");

    self_arc
  }

  pub(crate) async fn try_get_server(
    &self,
    server_id: u64,
  ) -> Result<&Arc<Mutex<DistrictServer>>, Box<dyn std::error::Error + Send + Sync>> {
    for server_lock in &self.servers {
      let server = server_lock.lock().await;
      if server.id == server_id {
        return Ok(server_lock);
      }
    }
    Err("Server not found".into())
  }

  pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
  }
}

#[async_trait]
impl<'r> FromRequest<'r> for &'r Application {
  type Error = std::convert::Infallible;

  async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
    let app_data = request.rocket().state::<Application>().unwrap();
    request::Outcome::Success(app_data)
  }
}
