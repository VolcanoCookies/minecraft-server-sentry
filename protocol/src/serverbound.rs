use packet::types::{VarInt, UUID};
use packet_macros::{Packet, PacketReadable, PacketWritable};

#[derive(PacketReadable, PacketWritable, Debug)]
#[packet(repr = "VarInt")]
pub enum HandshakeNextState {
    Status = 1,
    Login = 2,
    Transfer = 3,
}

#[derive(Packet, Debug)]
#[packet_id(0x00)]
#[packet_state(Handshaking)]
#[packet_direction(Serverbound)]
pub struct HandshakePacket {
    #[packet(repr = "VarInt")]
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: HandshakeNextState,
}

impl HandshakePacket {
    pub fn new(
        protocol_version: i32,
        server_address: &str,
        server_port: u16,
        next_state: HandshakeNextState,
    ) -> Self {
        Self {
            protocol_version: protocol_version,
            server_address: server_address.to_owned(),
            server_port,
            next_state,
        }
    }
}

#[derive(Packet, Debug)]
#[packet_id(0x00)]
#[packet_state(Status)]
#[packet_direction(Serverbound)]
pub struct StatusRequestPacket {}

impl StatusRequestPacket {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Packet, Debug)]
#[packet_id(0x01)]
#[packet_state(Status)]
#[packet_direction(Serverbound)]
pub struct PingRequestPacket {
    pub timestamp: i64,
}

impl PingRequestPacket {
    pub fn new(timestamp: i64) -> Self {
        Self { timestamp }
    }
}

#[derive(Packet, Debug)]
#[packet_id(0x01)]
#[packet_state(Login)]
#[packet_direction(Serverbound)]
pub struct EncryptionResponsePacket {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

impl EncryptionResponsePacket {
    pub fn new(shared_secret: Vec<u8>, verify_token: Vec<u8>) -> Self {
        Self {
            shared_secret,
            verify_token,
        }
    }
}

#[derive(Packet, Debug)]
#[packet_id(0x03)]
#[packet_state(Login)]
#[packet_direction(Serverbound)]
pub struct LoginAcknowledgedPacket {}

impl LoginAcknowledgedPacket {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Packet, Debug)]
#[packet_id(0x00)]
#[packet_state(Login)]
#[packet_direction(Serverbound)]
pub struct LoginStartPacket {
    pub name: String,
    pub uuid: UUID,
}

impl LoginStartPacket {
    pub fn new(name: &str, uuid: UUID) -> Self {
        Self {
            name: name.to_owned(),
            uuid,
        }
    }
}

pub mod configuration {
    use packet::types::VarInt;
    use packet_macros::{Packet, PacketReadable, PacketWritable};

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum ChatMode {
        Enabled = 0,
        CommandsOnly = 1,
        Hidden = 2,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum MainHand {
        Left = 0,
        Right = 1,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum ParticleStatus {
        All = 0,
        Decreased = 1,
        Minimal = 2,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x00)]
    #[packet_state(Configuration)]
    #[packet_direction(Serverbound)]
    pub struct ClientInformationPacket {
        pub locale: String,
        pub view_distance: i8,
        pub chat_mode: ChatMode,
        pub chat_colors: bool,
        pub displayed_skin_parts: u8,
        pub main_hand: MainHand,
        pub enable_text_filtering: bool,
        pub allow_server_listings: bool,
        pub particle_status: ParticleStatus,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x02)]
    #[packet_state(Configuration)]
    #[packet_direction(Serverbound)]
    pub struct ClientPluginMessagePacket {
        pub channel: String,
        #[packet(len = "rest")]
        pub data: Vec<u8>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x03)]
    #[packet_state(Configuration)]
    #[packet_direction(Serverbound)]
    pub struct FinishConfigurationPacket {}

    #[derive(Packet, Debug)]
    #[packet_id(0x04)]
    #[packet_state(Configuration)]
    #[packet_direction(Serverbound)]
    pub struct KeepAlivePacket {
        pub id: i64,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x07)]
    #[packet_state(Configuration)]
    #[packet_direction(Serverbound)]
    pub struct KnownPacksPacket {
        pub packs: Vec<crate::clientbound::configuration::Pack>,
    }
}

pub mod play {
    use packet::types::{fixed_bitset::FixedBitset, VarInt};
    use packet_macros::{Packet, PacketReadable, PacketWritable};

    #[derive(Packet, Debug)]
    #[packet_id(0x08)]
    #[packet_state(Play)]
    #[packet_direction(Serverbound)]
    pub struct ChatMessagePacket {
        pub message: String,
        pub timestamp: i64,
        pub salt: i64,
        pub signature: Option<[u8; 256]>,
        pub message_count: VarInt,
        pub acknowledged: FixedBitset<20>,
        pub checksum: u8,
    }

    impl ChatMessagePacket {
        
    }
}

#[cfg(test)]
mod tests {
    use packet::{raw::RawPacket, read::PacketReadable, write::PacketWritable, Packet, PacketData};

    use crate::serverbound::configuration::ChatMode;

    #[test]
    fn write_enum() {
        let mut buf = Vec::new();
        let enum_value = ChatMode::Hidden;

        PacketWritable::write(&enum_value, &mut buf).unwrap();
        assert_eq!(buf, vec![0x02]);
    }

    #[test]
    fn write_bool() {
        let mut buf = Vec::new();
        let value = true;

        PacketWritable::write(&value, &mut buf).unwrap();
        assert_eq!(buf, vec![0x01]);

        let mut buf = Vec::new();
        let value = false;

        PacketWritable::write(&value, &mut buf).unwrap();
        assert_eq!(buf, vec![0x00]);
    }

    #[test]
    fn write_login_start_packet() {
        let data = vec![
            0x1b, // Packet length
            0x00, // Packet ID
            0x09, // Name length
            0x56, 0x6f, 0x6c, 0x63, 0x61, 0x6e, 0x6f, 0x48, 0x44, // Name
            // UUID
            0x5c, 0xbe, 0xb7, 0x91, 0xfd, 0xf2, 0x49, 0x7a, 0xa6, 0x4a, 0x62, 0x90, 0x2b, 0x41,
            0x76, 0xe0,
        ];

        let mut read_buf = &data[..];
        let raw = RawPacket::read_blocking(&mut read_buf, -1).unwrap();
        let packet = super::LoginStartPacket::read(&mut raw.bytes.as_slice()).unwrap();
        assert_eq!(packet.name, "VolcanoHD");
        assert_eq!(
            packet.uuid.0,
            123279235262967524908973144513429206752u128.into()
        );

        let packet = super::LoginStartPacket::new(
            "VolcanoHD",
            123279235262967524908973144513429206752u128.into(),
        );
        let mut write_buf = Vec::new();
        packet.write(&mut write_buf).unwrap();
        let mut raw = RawPacket::new(packet.packet_id());
        raw.bytes = write_buf.clone();
        assert_eq!(data[2..], write_buf[..]);
        let mut write_buf = Vec::new();
        raw.write_blocking(&mut write_buf, -1).unwrap();
        assert_eq!(write_buf, data);
    }
}
