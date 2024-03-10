use std::io::{Cursor, Read};

use serde::{Deserialize, Serialize};
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::{
    model::uuid::UUID,
    packet_types::{PacketType, VarInt},
};

#[derive(Debug)]
pub struct Response {
    pub len: i32,
    pub packet_id: i32,
    pub data: ResponseData,
}

impl Response {
    pub async fn read(stream: &mut TcpStream) -> std::io::Result<Self> {
        //       let mut pre_buf = [0u8; 5];
        //        stream.read_exact(&mut pre_buf).await?;

        let len = VarInt::read_async(stream).await? as usize;

        let mut buf = vec![0; len];
        stream.read_exact(&mut buf).await?;

        let mut cursor = Cursor::new(&buf);

        let packet_id = VarInt::read(&mut cursor)?;
        let data_len = VarInt::read(&mut cursor)? as usize;
        let mut data_buf = vec![0; data_len];
        tokio::io::AsyncReadExt::read_exact(&mut cursor, &mut data_buf).await?;

        let response_data: ResponseData = serde_json::from_slice(data_buf.as_slice())?;

        /*
        if stream
            .peer_addr()
            .unwrap()
            .ip()
            .to_string()
            .eq("70.15.164.121")
        {
            let mut fout = File::options()
                .append(true)
                .create(true)
                .open("out.raw")
                .unwrap();
            fout.write(data_str.as_bytes());
            fout.write("\n".as_bytes());
        }
        */

        Ok(Self {
            len: len as i32,
            packet_id,
            data: response_data,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseData {
    pub version: Version,
    pub players: Players,
    pub favicon: Option<String>,
    #[serde(default = "default_bool_false")]
    #[serde(alias = "enforcesSecureChat")]
    pub enforces_secure_chat: bool,
    pub description: Description,
    #[serde(default = "default_string")]
    pub host: String,
    #[serde(default = "default_short")]
    pub port: i16,
    #[serde(alias = "forgeData")]
    pub forge_data: Option<ForgeData>,
}

fn default_bool_false() -> bool {
    false
}

fn default_string() -> String {
    "".to_owned()
}

fn default_short() -> i16 {
    0
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Players {
    pub max: i32,
    pub online: i32,
    #[serde(default = "default_list")]
    #[serde(alias = "list")]
    #[serde(alias = "sample")]
    pub list: Vec<Player>,
}

fn default_list() -> Vec<Player> {
    Vec::with_capacity(0)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub name: String,
    pub id: UUID,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Description {
    Raw(String),
    Nested {
        #[serde(alias = "text")]
        #[serde(alias = "translate")]
        text: String,
    },
}

impl Description {
    pub fn text(self) -> String {
        match self {
            Description::Raw(s) => s,
            Description::Nested { text } => text,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ForgeData {
    ForgeData {
        mods: Vec<Mod>,
        #[serde(alias = "fmlNetworkVersion")]
        fml_network_version: i32,
        #[serde(default = "default_bool_false")]
        truncated: bool,
    },
    ModInfo {
        #[serde(alias = "modList")]
        mod_list: Vec<Mod>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mod {
    #[serde(alias = "modId")]
    #[serde(alias = "modid")]
    pub mod_id: String,
    #[serde(alias = "modMarker")]
    #[serde(alias = "modmarker")]
    #[serde(alias = "version")]
    pub mod_marker: String,
}
