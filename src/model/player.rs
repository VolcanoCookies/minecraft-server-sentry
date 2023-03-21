use std::{
    hash::Hash,
    time::{SystemTime, UNIX_EPOCH},
};

use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

use crate::response::Player;

use super::uuid::UUID;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinecraftPlayer {
    pub uuid: UUID,
    pub name: String,
    pub last_seen: DateTime,
    pub last_updated: DateTime,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoricPlayer {
    pub uuid: UUID,
    pub last_seen: DateTime,
}

impl PartialEq for HistoricPlayer {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for HistoricPlayer {}

impl Hash for HistoricPlayer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnlinePlayer {
    pub uuid: UUID,
    pub online_since: DateTime,
}

impl From<Player> for OnlinePlayer {
    fn from(value: Player) -> Self {
        Self {
            uuid: value.id,
            online_since: DateTime::now(),
        }
    }
}

impl PartialEq for OnlinePlayer {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for OnlinePlayer {}

impl Hash for OnlinePlayer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}
