use std::io;
use std::net::SocketAddr;

use net::UdpSocket;
use packet::{RawPacket, Packet};

use super::Channel;

/// This channel receives data that has no guarantee that it is in ordered but that has control over dropped packets if there are any.
///
///  1. Reliable,
///  2. No guarantee for delivery
///  3. No guarantee that it is in order
///  4. Able to get dropped packets from the channel (udp with option to get dropped packets).
pub struct ReliableUnorderedChannel
{
    socket: UdpSocket
}

impl ReliableUnorderedChannel
{
    pub fn new(socket: UdpSocket) -> Self
    {
        ReliableUnorderedChannel { socket }
    }
}

impl Channel for ReliableUnorderedChannel
{
    fn send(&mut self, addr: SocketAddr, payload: &[u8]) -> io::Result<usize> {
        self.socket.send(Packet::new(addr, Vec::from(payload)))
    }

    fn recv(&mut self) -> io::Result<Option<Packet>> {
        self.socket.recv()
    }
}