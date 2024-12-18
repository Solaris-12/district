use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use serenity::all::PresenceData;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::logger::{LogLevel, Logger};
use crate::{log_d, log_x};

use super::application::Application;

pub async fn timer_loop(app: Arc<Mutex<Application>>) -> Result<(), String> {
  log_d!("Starting timer loop!");
  let mut timer_span: u64 = 0;
  loop {
    timer_span += 1;
    if timer_span % 20 == 0 {
      for server_lock in app.lock().await.servers.iter_mut() {
        let mut server = server_lock.lock().await;
        let mut presence_data: Option<PresenceData> = None;
        if server.status.last_heard.is_some() && server.status.open {
          let now = Utc::now().timestamp();
          let last_heard = server.status.last_heard.unwrap_or(0);
          let duration_since_last_heard = now - (last_heard as i64);
          if duration_since_last_heard > 500 {
            log_d!(format!(
              "Server {} did not reply in the last 500 seconds, setting player count to 0",
              server.id
            ));
            server.status.player_count = 0;
            server.status.player_ids = None;

            if duration_since_last_heard > 1000 {
              log_d!(format!(
                "Server {} did not reply in the last 1000 seconds, disconnecting server",
                server.id
              ));
              server.status.open = false;
              server.status.last_heard = None;
            }
          }

          if let Some(presence) = server
            .bot
            .as_ref()
            .and_then(|val| val.active_presence.clone())
          {
            presence_data = Some(PresenceData {
              activity: presence.activity.map(|mut val| {
                val.name = val
                  .name
                  .replace("{players}", &server.status.player_count.to_string())
                  .replace("{max_players}", &server.status.max_player_count.to_string());
                return val;
              }),
              status: presence.status,
            })
          }
        } else if let Some(presence) = server.bot.as_ref().and_then(|val| val.presence.clone()) {
          presence_data = Some(presence)
        }

        if let Some(bot) = server.bot.as_ref() {
          if let Some(presence) = presence_data {
            let ctx_option = bot.ctx_manager.get_ctx().await;
            if let Some(ctx_lock) = ctx_option {
              let ctx = ctx_lock.lock().await;
              ctx.set_presence(presence.activity, presence.status)
            }
          }
        }
        server.try_clear_buffer().await;
      }
    }

    if timer_span % 3600 == 0 {
      let app_ = app.lock().await;
      if let Some(config) = app_.config.as_ref() {
        if let Some(db_lock) = app_.databases.as_ref() {
          let mut db_handler = db_lock.lock().await;
          let strict_clear = config.databases.player_db_auto_clear_strict.unwrap_or(0);
          let normal_clear = config.databases.player_db_auto_clear_normal.unwrap_or(0);

          if strict_clear != 0 {
            let _ = db_handler
              .player_database
              .remove_inactive_players(strict_clear, true);
          }
          if normal_clear != 0 {
            let _ = db_handler
              .player_database
              .remove_inactive_players(normal_clear, false);
          }
        }
      }
    }

    sleep(Duration::from_secs(1)).await;
  }
}
