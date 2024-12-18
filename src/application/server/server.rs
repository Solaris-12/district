use std::{
  collections::{HashMap, VecDeque},
  sync::Arc,
  time::{Duration, Instant},
};

use serde::Serialize;
use serenity::all::ChannelId;
use tokio::sync::Mutex;

use super::status::DistrictServerStatus;
use crate::{
  application::{
    application::Application, bot::bot::DistrictBot, config::server::server_config::ConfigServer,
    routes::websocket::structs::WsResponses,
  },
  log_d, log_x,
  logger::{LogLevel, Logger},
};

#[derive(Debug, Clone)]
pub struct DistrictServer {
  pub(crate) app: Arc<Mutex<Application>>,
  pub(crate) id: u64,
  pub(crate) name: String,
  pub(crate) bot: Option<DistrictBot>,
  pub(crate) channel_id: String,
  pub status: DistrictServerStatus,
  pub(crate) ws_msgs: VecDeque<WsResponses>,
  buffer: VecDeque<String>,
  last_sent: Instant,
  pub(crate) srv_cfg: ConfigServer,
}

impl DistrictServer {
  pub async fn new(app: Arc<Mutex<Application>>, srv_cfg: ConfigServer) -> Arc<Mutex<Self>> {
    let cfg_clone = srv_cfg.clone();
    let server = DistrictServer {
      app: app,
      id: srv_cfg.id.clone(),
      name: srv_cfg.name.clone(),
      bot: None,
      channel_id: srv_cfg.channel_id,
      status: DistrictServerStatus::new(),
      ws_msgs: VecDeque::new(),
      buffer: VecDeque::new(),
      last_sent: Instant::now(),
      srv_cfg: cfg_clone,
    };

    log_d!(format!(
      "{:>3}: Creating sever '{}'",
      srv_cfg.id, srv_cfg.name
    ));
    let server_arc: Arc<Mutex<Self>> = Arc::new(Mutex::new(server));
    {
      let mut server_locked = server_arc.lock().await;
      let mut bot = DistrictBot::new(srv_cfg.bot, Some(Arc::clone(&server_arc)), None);
      bot.spawn(server_locked.name.clone()).await;
      server_locked.bot = Some(bot);
    }

    server_arc
  }

  pub async fn try_clear_buffer(&mut self) {
    if self.buffer.is_empty() {
      return;
    }
    if Instant::now().duration_since(self.last_sent) <= Duration::from_secs(2) {
      return;
    }
    let _ = self.send_message(String::new()).await;
  }

  pub async fn send_message(&mut self, data: String) -> Result<(), Box<dyn std::error::Error>> {
    if self.channel_id.is_empty() {
      return Ok(());
    }

    // Add the message to the buffer
    self.buffer.push_back(data);

    // Get the current time
    let now = Instant::now();

    if now.duration_since(self.last_sent) >= Duration::from_secs(2) {
      // Send all messages in the buffer
      let messages = self.buffer.clone();
      self.send_batch_message(messages).await?;
    }

    Ok(())
  }

  async fn send_batch_message(
    &mut self,
    mut messages: VecDeque<String>,
  ) -> Result<(), Box<dyn std::error::Error>> {
    let channel_id = self.channel_id.parse::<u64>().ok().map(ChannelId::from);
    if self.channel_id.is_empty() || messages.is_empty() || channel_id.is_none() {
      return Ok(());
    }
    let channel_id = channel_id.unwrap();

    // Remove duplicates
    let mut counts: HashMap<String, usize> = HashMap::new();
    for message in &messages {
      *counts.entry(message.clone()).or_insert(0) += 1;
    }

    messages = counts
      .into_iter()
      .map(|(k, v)| if v > 1 { format!("{} x{}", k, v) } else { k })
      .collect();

    messages.make_contiguous().sort_by(|a, b| {
      let a_chars: String = a.chars().take(14).collect();
      let b_chars: String = b.chars().take(14).collect();
      a_chars.cmp(&b_chars)
    });

    // Truncate
    if messages.len() > 20 {
      let start_index = messages.len().saturating_sub(15);
      messages = messages.split_off(start_index);
      log_d!("Truncated buffer");
    }

    // Combine all messages into one string
    let combined_message = Vec::from(messages).join("\n");
    if combined_message.is_empty() {
      return Ok(());
    }

    let now = Instant::now();
    self.buffer = VecDeque::new();
    self.last_sent = now;

    if let Some(bot) = self.bot.as_ref() {
      if let Some(context) = bot.ctx_manager.get_ctx().await {
        let ctx = context.lock().await;
        let message_content = MessageContent {
          content: combined_message,
        };

        ctx
          .http
          .send_message(channel_id, vec![], &message_content)
          .await
          .map_err(|e| format!("Discord Error {}", e))?;
      }
    }

    Ok(())
  }
}

#[derive(Serialize)]
struct MessageContent {
  content: String,
}
