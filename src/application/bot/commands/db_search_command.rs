use std::sync::Arc;

use serenity::all::{
  CacheHttp, Colour, CommandInteraction, Context, CreateCommand, CreateCommandOption, CreateEmbed,
  CreateEmbedAuthor, EditInteractionResponse, Permissions,
};
use tokio::sync::Mutex;

use crate::application::{
  application, db::player::structs::DatabasePlayer, server::server::DistrictServer,
};

use super::CommandHandler;

pub struct DbSearchCommand;

impl CommandHandler for DbSearchCommand {
  // MARK: Command handler
  async fn handle(
    &self,
    command: &CommandInteraction,
    ctx: &Context,
    server: Option<&Arc<Mutex<DistrictServer>>>,
  ) -> Result<(), String> {
    let http = ctx.http.clone();

    let search_by = command
      .data
      .options
      .get(0)
      .map(|val| val.value.as_i64().unwrap_or(0))
      .unwrap_or(0);
    let search_query = command
      .data
      .options
      .get(1)
      .map(|val| val.value.as_str().unwrap_or(""))
      .unwrap_or("");
    let search_exact = command
      .data
      .options
      .get(2)
      .map(|val| val.value.as_bool().unwrap_or(false))
      .unwrap_or(false);

    let _ = command.defer(http.clone()).await;

    if let Some(server_data) = server {
      let srv = server_data.lock().await;
      let app = srv.app.lock().await;
      if let Some(db_handler_arc) = app.databases.as_ref() {
        let db_handler = db_handler_arc.lock().await;
        let db = &db_handler.player_database;
        if let Ok(players) = db.get_all_players() {
          drop(db_handler);
          drop(app);
          let search_criterion = SearchCriterion::from_i64(search_by);
          let filtered_players = DbSearchCommand::search_by_criterion(
            players,
            search_criterion.clone(),
            search_query.to_string(),
            search_exact,
          );
          let _ = self
            .response_from_output(
              search_criterion,
              search_query.to_string(),
              search_exact,
              command,
              filtered_players,
              http,
            )
            .await;
          return Ok(());
        }
        drop(db_handler);
        drop(app);
        return self
          .send_error_msg(command, http, Some(r"SQL threw error"))
          .await;
      }
      return self
        .send_error_msg(command, http, Some(r"No databases loaded"))
        .await;
    }
    self
      .send_error_msg(command, http, Some(r"Server not found"))
      .await
  }

  // MARK: Command registration
  fn register(&self, permissions: Option<u64>) -> serenity::all::CreateCommand {
    CreateCommand::new("db_search")
      .description("[AT] Search the player database")
      .add_option(
        CreateCommandOption::new(
          serenity::all::CommandOptionType::Integer,
          "by",
          "By which column should be searched",
        )
        .add_int_choice("PlayerID", 0)
        .add_int_choice("SteamID", 1)
        .add_int_choice("Username", 2)
        .add_int_choice("IpAddr", 3)
        .required(true),
      )
      .add_option(
        CreateCommandOption::new(
          serenity::all::CommandOptionType::String,
          "query",
          "What to search",
        )
        .required(true),
      )
      .add_option(CreateCommandOption::new(
        serenity::all::CommandOptionType::Boolean,
        "exact",
        "If should check for exact equality of values",
      ))
      .default_member_permissions(Permissions::from_bits_truncate(
        permissions.unwrap_or(Permissions::MUTE_MEMBERS.bits()),
      ))
  }
}

impl DbSearchCommand {
  // MARK: Error message
  async fn send_error_msg(
    &self,
    command: &CommandInteraction,
    cache_http: impl CacheHttp,
    message: Option<impl ToString>,
  ) -> Result<(), String> {
    if let Some(msg) = message {
      command
        .edit_response(
          cache_http,
          EditInteractionResponse::new().content(format!(
            "There was an error when handling this command: {}!",
            msg.to_string()
          )),
        )
        .await
        .map_err(|e| e.to_string())
        .map(|_| ())
    } else {
      command
        .edit_response(
          cache_http,
          EditInteractionResponse::new().content("There was an error when handling this command!"),
        )
        .await
        .map_err(|e| e.to_string())
        .map(|_| ())
    }
  }

  // MARK: Search by
  fn search_by_criterion(
    data: Vec<DatabasePlayer>,
    criterion: SearchCriterion,
    search_query: String,
    exact: bool,
  ) -> Vec<DatabasePlayer> {
    data
      .into_iter()
      .filter(|player| match criterion {
        SearchCriterion::PlayerId => player.player_id.search_db(&search_query, exact),
        SearchCriterion::SteamId => player.steam_id.search_db(&search_query, exact),
        SearchCriterion::Usernames => player.usernames.search_db(&search_query, exact),
        SearchCriterion::Ips => player.ips.search_db(&search_query, exact),
        SearchCriterion::Unknown => false,
      })
      .collect()
  }

  // MARK: Create response with output
  async fn response_from_output(
    &self,
    search_by: SearchCriterion,
    search_query: String,
    search_exact: bool,
    command: &CommandInteraction,
    data: Vec<DatabasePlayer>,
    cache_http: impl CacheHttp,
  ) -> Result<(), String> {
    let search_input = format!(
      "DISTRICT search:\n- **Search by**: {}\n- **Search query**: _{}_\n- **Search exact?**: {}",
      search_by.to_string(),
      search_query,
      if search_exact { "Yes" } else { "No" },
    );
    if data.is_empty() {
      command
        .edit_response(
          cache_http,
          EditInteractionResponse::new().content(format!("{}\nFound nothing...", search_input)),
        )
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
    } else if data.len() == 1 {
      let player: &DatabasePlayer = &data[0];
      let player_usernames: String = player
        .usernames
        .clone()
        .into_iter()
        .rev()
        .take(10)
        .collect::<Vec<_>>()
        .join("\n");
      let player_ips: String = player
        .ips
        .clone()
        .into_iter()
        .rev()
        .take(10)
        .map(|val| format!("||{}||", val))
        .collect::<Vec<_>>()
        .join("\n");
      command.edit_response(
                cache_http,
                EditInteractionResponse::new()
                    .content(search_input)
                    .add_embed(
                        CreateEmbed::new()
                            .author(
                                CreateEmbedAuthor::new(format!("Player {}", player.player_id))
                                    .icon_url(application::APPLICATION_ICON_URL),
                            )
                            .title(&player.steam_id)
                            .url(format!(
                                "https://steamcommunity.com/profiles/{}/",
                                player.steam_id.clone().split_once('@').map_or("", |(first, _)| first)
                            ))
                            .color(Colour::from_rgb(152, 212, 245))
                            .thumbnail(application::APPLICATION_ICON_URL)
                            .description(
                                format!("- **First join**: <t:{}:R>\n- **Last join**: <t:{}:R>\n- **Times joined**: {}\n- **Hours played**: {}\n- **Do Not Track**: {}\n- **Is Verified**: {}",
                                    player.first_join_date.timestamp(),
                                    player.last_join_date.timestamp(),
                                    player.times_joined,
                                    player.hours_played,
                                    if player.do_not_track { "Yes" } else { "No" },
                                    if player.is_verified() { "Yes" } else { "No" },
                                )
                            ).field("Usernames:", player_usernames, false).field("Ips:", player_ips, false),
                    ),
            ).await.map(|_| ())
            .map_err(|e| e.to_string())
    } else {
      command
        .edit_response(
          cache_http,
          EditInteractionResponse::new()
            .content(search_input)
            .add_embeds(
              data
                .iter()
                .take(10)
                .map(|player| {
                  CreateEmbed::new()
                    .author(
                      CreateEmbedAuthor::new(format!("Player {}", player.player_id))
                        .icon_url(application::APPLICATION_ICON_URL),
                    )
                    .title(player.steam_id.clone())
                    .color(Colour::from_rgb(152, 212, 245))
                    .description(format!(
                      "- Last username: {}",
                      player.usernames.last().unwrap_or(&String::new())
                    ))
                })
                .collect::<Vec<_>>(),
            ),
        )
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
    }
  }
}

// MARK: Search impl
#[derive(Clone)]
pub(super) enum SearchCriterion {
  PlayerId,
  SteamId,
  Usernames,
  Ips,
  Unknown,
}

impl ToString for SearchCriterion {
  fn to_string(&self) -> String {
    match self {
      SearchCriterion::PlayerId => "PlayerId".to_string(),
      SearchCriterion::SteamId => "SteamId".to_string(),
      SearchCriterion::Usernames => "Usernames".to_string(),
      SearchCriterion::Ips => "Ips".to_string(),
      SearchCriterion::Unknown => "Unknown".to_string(),
    }
  }
}

impl SearchCriterion {
  fn from_i64(n: i64) -> Self {
    match n {
      0 => Self::PlayerId,
      1 => Self::SteamId,
      2 => Self::Usernames,
      3 => Self::Ips,
      _ => Self::Unknown,
    }
  }
}

trait Searchable {
  fn search_db(&self, query: &str, exact: bool) -> bool;
}

impl Searchable for String {
  fn search_db(&self, query: &str, exact: bool) -> bool {
    if exact {
      self.to_string() == query
    } else {
      self.to_string().contains(query)
    }
  }
}

impl Searchable for u64 {
  fn search_db(&self, query: &str, exact: bool) -> bool {
    if exact {
      self.to_string() == query
    } else {
      self.to_string().contains(query)
    }
  }
}

impl Searchable for Vec<String> {
  fn search_db(&self, query: &str, exact: bool) -> bool {
    if exact {
      self.into_iter().any(|val| val == query)
    } else {
      self.into_iter().any(|val| val.contains(query))
    }
  }
}
