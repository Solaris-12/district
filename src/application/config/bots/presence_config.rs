use std::str::FromStr;

use reqwest::Url;
use serde::{Deserialize, Serialize};
use serenity::all::ActivityData;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PresenceConfig {
  pub status: String,
  pub kind: u8,
  pub name: String,
  pub url: String,
}

impl PresenceConfig {
  pub fn into_presence(&self) -> serenity::all::PresenceData {
    serenity::all::PresenceData {
      activity: Some(ActivityData {
        name: self.name.clone(),
        kind: self.kind.into(),
        state: Some(self.name.clone()),
        url: Url::from_str(self.url.as_str()).ok(),
      }),
      status: serenity::all::OnlineStatus::from(
        LocalOnlineStatus::from_str(self.status.as_str())
          .unwrap()
          .into(),
      ),
    }
  }
}

pub enum LocalOnlineStatus {
  DoNotDisturb,
  Idle,
  Invisible,
  Offline,
  Online,
}

impl FromStr for LocalOnlineStatus {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "dnd" => Ok(LocalOnlineStatus::DoNotDisturb),
      "idle" => Ok(LocalOnlineStatus::Idle),
      "invisible" => Ok(LocalOnlineStatus::Invisible),
      "offline" => Ok(LocalOnlineStatus::Offline),
      _ => Ok(LocalOnlineStatus::Online),
    }
  }
}

impl Into<serenity::all::OnlineStatus> for LocalOnlineStatus {
  fn into(self) -> serenity::all::OnlineStatus {
    match self {
      LocalOnlineStatus::DoNotDisturb => serenity::all::OnlineStatus::DoNotDisturb,
      LocalOnlineStatus::Idle => serenity::all::OnlineStatus::Idle,
      LocalOnlineStatus::Invisible => serenity::all::OnlineStatus::Invisible,
      LocalOnlineStatus::Offline => serenity::all::OnlineStatus::Offline,
      LocalOnlineStatus::Online => serenity::all::OnlineStatus::Online,
    }
  }
}
