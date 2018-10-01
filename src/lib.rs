//! Amethysts networking protocol

extern crate bincode;
extern crate serde;
extern crate byteorder;
#[macro_use]
extern crate serde_derive;

mod net;
mod packet;

pub use net::UdpSocket;
use packet::{Packet};
