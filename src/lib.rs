//! Amethysts networking protocol

extern crate bincode;
extern crate serde;

#[macro_use]
extern crate serde_derive;

mod net;
mod packet;
mod channel;

pub use net::UdpSocket;
use packet::{Packet, RawPacket};
