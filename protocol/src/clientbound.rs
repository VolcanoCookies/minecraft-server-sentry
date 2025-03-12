use packet::types::{VarInt, UUID};
use packet_macros::{Packet, PacketReadable, PacketWritable};
use serde::{Deserialize, Serialize};

#[derive(Packet, Debug)]
#[packet_id(0x01)]
#[packet_state(Login)]
#[packet_direction(Clientbound)]
pub struct EncryptionRequestPacket {
    pub server_id: String,
    pub public_key: Vec<u8>,
    pub verify_token: Vec<u8>,
    pub should_authenticate: bool,
}

#[derive(Packet, Debug)]
#[packet_id(0x02)]
#[packet_state(Login)]
#[packet_direction(Clientbound)]
pub struct LoginSuccessPacket {
    pub uuid: UUID,
    pub username: String,
    pub number_of_properties: i32,
    pub properties: Vec<Property>,
}

#[derive(PacketReadable, PacketWritable, Debug)]
pub struct Property {
    pub name: String,
    pub value: String,
    pub is_signed: bool,
    pub signature: Option<String>,
}

#[derive(Packet, Debug)]
#[packet_id(0x00)]
#[packet_state(Status)]
#[packet_direction(Clientbound)]
pub struct StatusResponsePacket {
    pub json: StatusResponseJson,
}

#[derive(Debug, Serialize, Deserialize, PacketReadable, PacketWritable)]
#[packet(repr = "json")]
#[serde(rename_all = "camelCase")]
pub struct StatusResponseJson {
    version: Version,
    players: Option<Players>,
    description: Option<Description>,
    favicon: Option<String>,
    enforce_secure_chat: Option<bool>,
    prevent_chat_reports: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    name: String,
    protocol: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Players {
    max: i32,
    online: i32,
    #[serde(default)]
    sample: Vec<PlayerSample>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerSample {
    name: String,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Description {
    Text { text: String },
    String(String),
}

#[derive(Packet, Debug)]
#[packet_id(0x03)]
#[packet_state(Login)]
#[packet_direction(Clientbound)]
pub struct SetCompressionPacket {
    #[packet(repr = "VarInt")]
    pub threshold: i32,
}

#[derive(Packet, Debug)]
#[packet_id(0x03)]
#[packet_state(Configuration)]
#[packet_direction(Clientbound)]
pub struct FinishConfigurationPacket {}

#[derive(Packet, Debug)]
#[packet_id(0x00)]
#[packet_state(Login)]
#[packet_direction(Clientbound)]
pub struct DisconnectPacket {
    pub reason: String,
}
