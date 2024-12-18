use crate::application::bot::commands::db_search_command::DbSearchCommand;
use crate::application::bot::commands::info_command::InfoCommand;
use crate::application::bot::commands::send_command::SendCommand;
use crate::application::bot::commands::{CommandHandler as _, CommandHandlerEnum};
use crate::application::config::bots::BotConfig;
use crate::logger::{LogLevel, Logger};
use crate::{log_e, log_i, log_x};
use serenity::all::{
  CreateInteractionResponse, CreateInteractionResponseMessage, Interaction, Ready,
};
use serenity::client::Context;
use serenity::prelude::EventHandler;

use super::bot::DistrictBot;

#[async_trait]
impl EventHandler for DistrictBot {
  async fn ready(&self, ctx: Context, ready: Ready) {
    log_i!(format!("Logged as {} ({})", ready.user.name, ready.user.id));
    if let Some(presence) = &self.presence {
      ctx.set_presence(presence.activity.clone(), presence.status);
    }

    if let Some(info_permissions) = match &self.bot_config {
      BotConfig::ConfigBot(cfg) => cfg.commands.info_command,
      BotConfig::ServerBotConfig(cfg) => cfg.commands.info_command,
    } {
      let _ = self
        .operational_guild
        .create_command(&ctx.http, InfoCommand.register(Some(info_permissions)))
        .await
        .map_err(|e| log_e!(e));
    }
    if let Some(search_permissions) = match &self.bot_config {
      BotConfig::ConfigBot(cfg) => cfg.commands.db_search,
      BotConfig::ServerBotConfig(cfg) => cfg.commands.db_search,
    } {
      let _ = self
        .operational_guild
        .create_command(
          &ctx.http,
          DbSearchCommand.register(Some(search_permissions)),
        )
        .await
        .map_err(|e| log_e!(e));
    }
    if let Some(send_permissions) = match &self.bot_config {
      BotConfig::ConfigBot(cfg) => cfg.commands.send_command,
      BotConfig::ServerBotConfig(cfg) => cfg.commands.send_command,
    } {
      let _ = self
        .operational_guild
        .create_command(&ctx.http, SendCommand.register(Some(send_permissions)))
        .await
        .map_err(|e| log_e!(e));
    }

    self.ctx_manager.set_ctx(ctx).await;
  }
  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    if let Interaction::Command(command) = interaction {
      // find command handler
      let command_handler: Result<CommandHandlerEnum, String> = match command.data.name.as_str() {
        "info" => Ok(CommandHandlerEnum::InfoCommand(InfoCommand)),
        "db_search" => Ok(CommandHandlerEnum::DbSearchCommand(DbSearchCommand)),
        "send_command" => Ok(CommandHandlerEnum::SendCommand(SendCommand)),
        _ => return,
      };

      // execute command handler
      match command_handler {
        Ok(handler) => {
          if let Err(error_msg) = handler.handle(&command, &ctx, self.server.as_ref()).await {
            let _ = command
              .create_response(
                ctx.http,
                CreateInteractionResponse::Message(
                  CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content(error_msg),
                ),
              )
              .await
              .map_err(|e| log_e!(e));
          }
        }
        Err(_) => return,
      }
    }
  }
}
