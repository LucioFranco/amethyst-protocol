use std::net::{SocketAddr, IpAddr};
use std::io;
use std::str::{self, FromStr};

mod reliable_unordered;
//mod reliable;
mod unreliable;
mod channel;

pub use self::reliable_unordered::ReliableUnorderedChannel;
pub use self::unreliable::UnreliableChannel;
pub use self::channel::CommunicationChannel;

use packet::Packet;

pub enum ChannelType
{

    /// This type means:
    ///
    ///  1. Reliable.
    ///  2. No guarantee for delivery.
    ///  3. No guarantee for order.
    ///  4. Able to get dropped packets from the channel (udp with option to get dropped packets).
    ///
    /// Basically UDP but with a way to retrieve dropped packets.
    ReliableUnordered,
    /// This type means:
    ///
    /// 1. Unreliable
   ///  2. No guarantee for delivery.
   ///  3. No guarantee for order.
   ///  4. No way of getting dropped packet
   ///
   /// Basically just bare UDP
    Unreliable,

    // TODO: Reliable, guarantee for delivery (just bare tcp)
    // Reliable,
    // Todo: Unreliable, this represents an channel that will deliver everything in order. If packets do not arrive they will be stored and you can decide what to do with them.
    // Sequenced,
}

/// This trait provides an interface for an chanel that could be used to communicate over.
pub trait Channel
{
    /// Send information to the given endpoint.
    fn send(&mut self, addr: SocketAddr, payload: &[u8])  -> io::Result<usize>;
    /// Receive data from the channel and return the packet if there is result.
    fn recv(&mut self) ->  io::Result<Option<Packet>>;
}







