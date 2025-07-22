use packet::types::VarInt;
use packet_macros::{PacketReadable, PacketWritable};

#[derive(PacketReadable, PacketWritable, Debug)]
#[packet(index = "VarInt")]
pub enum EnumStruct {
    AddBlock { x: i32, y: i32, z: i32 },
    RemoveBlock { id: VarInt },
    GetBlockInfo { entity_id: VarInt, name: String },
}

fn main() {
    println!("Hello, world!");
}
