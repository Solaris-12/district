use std::sync::Arc;

use serenity::all::{
  Colour, CommandInteraction, Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
  CreateInteractionResponseMessage, Permissions,
};
use tokio::sync::Mutex;

use crate::application::server::server::DistrictServer;

use super::CommandHandler;

pub struct InfoCommand;

impl CommandHandler for InfoCommand {
  async fn handle(
    &self,
    command: &CommandInteraction,
    ctx: &Context,
    server: Option<&Arc<Mutex<DistrictServer>>>,
  ) -> Result<(), String> {
    let http = ctx.http.clone();
    if let Some(server_data) = server {
      let srv = server_data.lock().await;
      command
                .create_response(
                    http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().embed(
                            CreateEmbed::new()
                                .color(Colour::from_rgb(126, 212, 212))
                                .title(format!("DISTRICT server '{}'", srv.name))
                                .description("This is a District server bot!\nIt's used to control the server and interact with the players.\nMost of these interactions are only available to the server administrators."),
                        ),
                    ),
                )
                .await
                .map_err(|e| e.to_string())
    } else {
      command
                .create_response(
                    http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().embed(
                            CreateEmbed::new()
                                .color(Colour::from_rgb(126, 212, 212))
                                .title(format!("DISTRICT"))
                                .description("This is a District server bot!\nIt's used to control the server and interact with the players.\nMost of these interactions are only available to the server administrators."),
                        ),
                    ),
                )
                .await
                .map_err(|e| e.to_string())
    }
  }
  fn register(&self, permissions: Option<u64>) -> serenity::all::CreateCommand {
    CreateCommand::new("info")
      .description("Gives information about the server!")
      .default_member_permissions(Permissions::from_bits_truncate(
        permissions.unwrap_or(Permissions::empty().bits()),
      ))
  }
}
