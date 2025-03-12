use std::hash::Hash;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use welds::WeldsModel;

use crate::model::uuid::UUID;

use super::server::MinecraftServer;

#[derive(Debug, Serialize, Deserialize, Clone, WeldsModel)]
#[welds(table = "minecraft_players")]
#[welds(HasMany(history, HistoricPlayer, "uuid"))]
pub struct MinecraftPlayer {
    #[welds(primary_key)]
    pub uuid: UUID,
    pub username: String,
    pub last_seen: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

impl PartialEq for MinecraftPlayer {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for MinecraftPlayer {}

impl Hash for MinecraftPlayer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, WeldsModel)]
#[welds(table = "minecraft_player_history")]
#[welds(BelongsTo(player, MinecraftPlayer, "player"))]
#[welds(BelongsTo(server, MinecraftServer, "server"))]
pub struct HistoricPlayer {
    #[welds(primary_key)]
    pub id: UUID,
    pub player: UUID,
    pub server: UUID,
    pub last_seen: DateTime<Utc>,
}

impl PartialEq for HistoricPlayer {
    fn eq(&self, other: &Self) -> bool {
        self.player == other.player
    }
}

impl Eq for HistoricPlayer {}

impl Hash for HistoricPlayer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.player.hash(state);
    }
}
