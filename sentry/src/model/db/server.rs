use chrono::DateTime;
use serde::{Deserialize, Serialize};
use welds::WeldsModel;

use crate::model::uuid::UUID;

use super::player::HistoricPlayer;

#[derive(Debug, Serialize, Deserialize, Clone, WeldsModel)]
#[welds(table = "minecraft_servers")]
#[welds(HasMany(player_history, HistoricPlayer, "server"))]
pub struct MinecraftServer {
    #[welds(primary_key)]
    pub id: UUID,
    pub host: String,
    pub port: i16,
    pub whitelist: bool,
    pub max_players: i32,
    pub players: i32,
    pub motd: String,
    pub version_name: String,
    pub version_protocol: i32,
    pub last_updated: DateTime<chrono::Utc>,
}

impl MinecraftServer {
    pub fn version(&self) -> Version {
        Version {
            name: self.version_name.clone(),
            protocol: self.version_protocol,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
}
