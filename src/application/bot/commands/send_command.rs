use std::sync::Arc;

use serenity::all::{
  CommandInteraction, Context, CreateCommand, CreateCommandOption, CreateInteractionResponse,
  CreateInteractionResponseMessage, Permissions,
};
use tokio::sync::Mutex;

use crate::application::{
  routes::websocket::structs::{WsCommandResponse, WsResponseCreate},
  server::server::DistrictServer,
};

use super::CommandHandler;

pub struct SendCommand;

impl CommandHandler for SendCommand {
  async fn handle(
    &self,
    command: &CommandInteraction,
    ctx: &Context,
    server: Option<&Arc<Mutex<DistrictServer>>>,
  ) -> Result<(), String> {
    let http = ctx.http.clone();
    let cmd = command
      .data
      .options
      .get(0)
      .map(|val| val.value.as_str().unwrap_or(""))
      .unwrap_or("");
    if let Some(srv_lock) = server {
      let mut srv = srv_lock.lock().await;
      srv.ws_msgs.push_back(WsCommandResponse::create(
        crate::application::routes::websocket::structs::WsReponseStatus::Ok,
        "Ok",
        Some(cmd),
      ));
      let _ = command
        .create_response(
          http,
          CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
              .content("Command sent!")
              .ephemeral(true),
          ),
        )
        .await;
    }
    Ok(())
  }
  fn register(&self, permissions: Option<u64>) -> serenity::all::CreateCommand {
    CreateCommand::new("send_command")
      .description("[AT] Sends a command message to the specified server")
      .default_member_permissions(Permissions::ADMINISTRATOR)
      .add_option(
        CreateCommandOption::new(
          serenity::all::CommandOptionType::String,
          "command",
          "Command that will be sent to the server",
        )
        .required(true),
      )
      .default_member_permissions(Permissions::from_bits_truncate(
        permissions.unwrap_or(Permissions::ADMINISTRATOR.bits()),
      ))
  }
}
