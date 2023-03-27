use std::{error::Error, fs::File, io::Write};

use serde::{Deserialize, Serialize};
use serde_json::Result;
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::{model::uuid::UUID, packet, types::VarInt};

#[derive(Debug)]
pub struct Response {
    pub len: i32,
    pub packet_id: i32,
    pub data: ResponseData,
}

impl Response {
    pub async fn read(stream: &mut TcpStream) -> std::io::Result<Self> {
        let mut pre_buf = [0u8; 5];
        stream.read_exact(&mut pre_buf).await?;

        let mut pre_buf_iter = pre_buf.iter();
        let len = VarInt::parse(&mut pre_buf_iter) as usize;
        let extra = pre_buf_iter.len();

        let mut rest = vec![0; len - extra];
        stream.read_exact(&mut rest).await?;

        let mut buf = Vec::<u8>::new();
        buf.append(&mut pre_buf_iter.cloned().collect::<Vec<u8>>());
        buf.append(&mut rest);

        let mut buf_iter = buf.iter();
        let packet_id = VarInt::parse(&mut buf_iter);
        // The rest of the packet is a string so we need to read the length of it but can ignore it
        VarInt::parse(&mut buf_iter);
        let data: Vec<u8> = buf_iter.cloned().collect();

        /*
        let data_str = std::string::String::from_utf8_lossy(&data);

        let response_data_result: Result<ResponseData> = serde_json::from_str(&data_str);

        let response_data = match response_data_result {
            Ok(d) => d,
            Err(error) => {
                let err_file = File::create(stream.peer_addr().unwrap().ip().to_string());
                err_file.unwrap().write_all(&data);
                panic!("{}", error);
            }
        };
         */

        let response_data: ResponseData = serde_json::from_slice(data.as_slice())?;

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
