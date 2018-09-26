use net::Connection;
use std::net::SocketAddr;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Packet {
    // the address to witch the packet will be send
    pub addr: SocketAddr,
    // the raw payload of the packet
    pub payload: Box<[u8]>,
}

impl Packet {
    pub fn new(addr: SocketAddr, payload: Vec<u8>) -> Self {
        Packet {
            addr,
            payload: payload.into_boxed_slice(),
        }
    }

    pub fn payload(&self) -> &[u8] {
        return &self.payload;
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}