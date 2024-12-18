use std::sync::Arc;

use serenity::{
  all::{Context, GatewayIntents, GuildId, PresenceData},
  Client,
};
use tokio::sync::Mutex;

use crate::{
  application::config::bots::BotConfig,
  logger::{LogLevel, Logger},
};
use crate::{application::server::server::DistrictServer, log_d, log_e, log_x};

#[derive(Debug, Clone)]
pub struct DistrictBot {
  pub(super) server: Option<Arc<Mutex<DistrictServer>>>,
  pub(crate) token: String,
  pub(crate) presence: Option<PresenceData>,
  pub(crate) active_presence: Option<PresenceData>,
  pub(super) operational_guild: GuildId,
  pub(crate) ctx_manager: Arc<ContextManager>,
  pub(crate) bot_config: BotConfig,
}

impl DistrictBot {
  pub fn new(
    bot_cfg: BotConfig,
    server: Option<Arc<Mutex<DistrictServer>>>,
    presence: Option<PresenceData>,
  ) -> Self {
    if bot_cfg.get_token().is_empty() {
      log_e!("Bot token is empty, check the config");
      panic!("Bot token is empty, check the config");
    }

    if bot_cfg.get_operational_guild_id() <= 10000000000000000 {
      log_e!("Operational GuildID has to be at least 17 digits long, check the config and set it to discord server ID");
      panic!("Operational GuildID has to be at least 17 digits long, check the config and set it to discord server ID");
    }

    match bot_cfg.clone() {
      BotConfig::ConfigBot(cfg) => {
        log_d!("Creating bot with config bot");
        return DistrictBot {
          bot_config: bot_cfg.clone(),
          server: server,
          token: cfg.token.clone(),
          presence: presence.or(cfg.default_presence.as_ref().map(|val| val.into_presence())),
          active_presence: None,
          operational_guild: cfg.active_guild_id.into(),
          ctx_manager: Arc::new(ContextManager::new()),
        };
      }
      BotConfig::ServerBotConfig(cfg) => {
        log_d!("Creating bot with server bot config");
        return DistrictBot {
          bot_config: bot_cfg.clone(),
          server: server,
          token: cfg.token.clone(),
          presence: presence.or(cfg.default_presence.as_ref().map(|val| val.into_presence())),
          active_presence: cfg.default_presence.as_ref().map(|val| val.into_presence()),
          operational_guild: cfg.active_guild_id.into(),
          ctx_manager: Arc::new(ContextManager::new()),
        };
      }
    };
  }
  pub async fn spawn(&mut self, server_name: String) {
    let token = self.token.clone();
    let intents = GatewayIntents::non_privileged()
      | GatewayIntents::MESSAGE_CONTENT
      | GatewayIntents::GUILDS
      | GatewayIntents::GUILD_MESSAGES
      | GatewayIntents::GUILD_MESSAGE_REACTIONS
      | GatewayIntents::DIRECT_MESSAGES
      | GatewayIntents::DIRECT_MESSAGE_REACTIONS;

    let self_arc = self.clone();
    let mut client = match Client::builder(&token, intents)
      .event_handler(self_arc)
      .await
    {
      Ok(val) => val,
      Err(e) => {
        log_e!(format!("Failed to create client: {}", e));
        panic!("Failed to create client");
      }
    };

    log_d!(format!("Spawning discord bot for '{}' server", server_name));
    tokio::spawn(async move {
      if let Err(why) = client.start().await {
        log_e!(format!("Client error: {}", why))
      }
    });
  }
}

#[derive(Debug, Clone)]
pub struct ContextManager {
  ctx: Arc<Mutex<Option<Context>>>,
}

impl ContextManager {
  pub fn new() -> Self {
    ContextManager {
      ctx: Arc::new(Mutex::new(None)),
    }
  }

  pub async fn set_ctx(&self, ctx: Context) {
    let mut ctx_guard = self.ctx.lock().await;
    *ctx_guard = Some(ctx);
  }

  pub async fn get_ctx(&self) -> Option<Arc<Mutex<Context>>> {
    let ctx_guard = self.ctx.lock().await;
    Some(Arc::new(Mutex::new(ctx_guard.clone().unwrap())))
  }
}
