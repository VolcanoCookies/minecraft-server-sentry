use std::collections::HashSet;

use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

use crate::response::Version;

use super::player::{HistoricPlayer, OnlinePlayer};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinecraftServer {
    pub host: String,
    pub port: i16,
    pub whitelist: bool,
    pub online: Online,
    pub historic_players: HashSet<HistoricPlayer>,
    pub motd: String,
    pub version: Version,
    pub last_updated: DateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Online {
    pub max: i32,
    pub players: i32,
    pub list: HashSet<OnlinePlayer>,
}
