use linkme::distributed_slice;

use crate::{ConnectionState, PacketDirection};

#[distributed_slice]
pub static PACKET_REGISTRY: [PacketDescriptor];

pub fn get_packet(
    id: i32,
    state: ConnectionState,
    direction: PacketDirection,
) -> Option<&'static PacketDescriptor> {
    PACKET_REGISTRY.iter().find(|descriptor| {
        descriptor.id == id && descriptor.state == state && descriptor.direction == direction
    })
}

#[derive(Debug, Clone)]
pub struct PacketDescriptor {
    pub name: &'static str,
    pub id: i32,
    pub state: crate::ConnectionState,
    pub direction: crate::PacketDirection,
    pub read_fn: fn(&mut std::io::Read) -> std::io::Result<Box<dyn crate::Packet>>,
}
