pub(super) mod http;
pub(super) mod messages;
pub mod websocket;

use http::log_routes::{log_server_status, log_with_translation};
use http::r#static::{static_index_page, static_test};
use rocket::Route;

use self::http::db::leaderboard::{
  db_add_stat_to_players_leaderboards, db_clear_leaderboards, db_get_player_leaderboards,
  db_leaderboards_remove_by_date,
};
use self::http::db::{
  players::{
    db_add_punishment_to_player, db_get_all_players, db_get_player_by_discord_id,
    db_get_player_by_player_id, db_get_player_by_steam_id, db_get_player_count, db_on_player_join,
    db_set_some_player_count,
  },
  punishments::{
    db_get_all_punishments, db_get_punishment_by_punishment_id, get_punishments_by_ip,
    get_punishments_by_player_id, get_punishments_by_steam_id,
  },
  verification::{
    db_add_player_verification, db_get_player_verification_by_code,
    db_get_player_verification_by_discord_id, db_get_player_verification_by_player_id,
    db_get_player_verification_by_steam_id, db_update_player_verification,
    dn_modify_player_verification,
  },
};
use self::websocket::websocket_connect;

#[derive(Debug, Clone)]
pub(crate) struct DistrictRouter {
  routes: Vec<Route>,
}

impl DistrictRouter {
  pub fn new() -> Self {
    DistrictRouter {
      routes: routes![
        static_index_page,
        static_test,
        log_server_status,
        log_with_translation,
        websocket_connect,
        db_on_player_join,
        db_get_all_players,
        db_get_player_by_player_id,
        db_get_player_by_steam_id,
        db_get_player_by_discord_id,
        db_add_punishment_to_player,
        db_get_player_count,
        db_set_some_player_count,
        db_get_all_punishments,
        db_get_punishment_by_punishment_id,
        get_punishments_by_player_id,
        get_punishments_by_steam_id,
        get_punishments_by_ip,
        db_get_player_verification_by_player_id,
        db_get_player_verification_by_steam_id,
        db_get_player_verification_by_discord_id,
        db_get_player_verification_by_code,
        db_add_player_verification,
        db_update_player_verification,
        dn_modify_player_verification,
        db_get_player_leaderboards,
        db_add_stat_to_players_leaderboards,
        db_clear_leaderboards,
        db_leaderboards_remove_by_date,
      ],
    }
  }
  pub fn get_routes(&self) -> &Vec<Route> {
    &self.routes
  }
}
