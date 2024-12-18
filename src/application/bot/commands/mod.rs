use std::sync::Arc;

use serenity::all::{CommandInteraction, Context, CreateCommand};
use tokio::sync::Mutex;

use crate::application::server::server::DistrictServer;

pub(crate) mod db_search_command;
pub(crate) mod info_command;
pub(crate) mod send_command;

pub trait CommandHandler: Send {
    fn handle(
        &self,
        command: &CommandInteraction,
        ctx: &Context,
        server: Option<&Arc<Mutex<DistrictServer>>>,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
    fn register(&self, permissions: Option<u64>) -> CreateCommand;
}

pub enum CommandHandlerEnum {
    InfoCommand(self::info_command::InfoCommand),
    DbSearchCommand(self::db_search_command::DbSearchCommand),
    SendCommand(self::send_command::SendCommand),
}

impl CommandHandler for CommandHandlerEnum {
    async fn handle(
        &self,
        command: &CommandInteraction,
        ctx: &Context,
        server: Option<&Arc<Mutex<DistrictServer>>>,
    ) -> Result<(), String> {
        match self {
            CommandHandlerEnum::InfoCommand(handler) => handler.handle(command, ctx, server).await,
            CommandHandlerEnum::DbSearchCommand(handler) => {
                handler.handle(command, ctx, server).await
            }
            CommandHandlerEnum::SendCommand(handler) => handler.handle(command, ctx, server).await,
        }
    }
    fn register(&self, permissions: Option<u64>) -> CreateCommand {
        match self {
            CommandHandlerEnum::InfoCommand(handler) => handler.register(permissions),
            CommandHandlerEnum::DbSearchCommand(handler) => handler.register(permissions),
            CommandHandlerEnum::SendCommand(handler) => handler.register(permissions),
        }
    }
}
