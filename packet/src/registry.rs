use std::{fmt::Debug, hash::Hash};

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

#[derive(Clone)]
pub struct PacketDescriptor {
    pub name: &'static str,
    pub id: i32,
    pub state: crate::ConnectionState,
    pub direction: crate::PacketDirection,
    pub read_fn: fn(&[u8]) -> std::io::Result<Box<dyn crate::PacketData>>,
    pub write_fn: fn(Box<dyn crate::PacketData>, &mut [u8]) -> std::io::Result<()>,
}

impl Debug for PacketDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketDescriptor")
            .field("name", &self.name)
            .field("id", &self.id)
            .field("state", &self.state)
            .field("direction", &self.direction)
            .finish()
    }
}

impl PartialEq for PacketDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.state == other.state && self.direction == other.direction
    }
}

impl Hash for PacketDescriptor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.state.hash(state);
        self.direction.hash(state);
    }
}
