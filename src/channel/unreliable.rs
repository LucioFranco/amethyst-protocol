use std::net::{UdpSocket, SocketAddr};
use std::io;

use packet::{RawPacket, Packet};
use super::Channel;

use net::constants::UDP_BUFFER_SIZE;

/// This channel receives data that has no guarantee that it is in ordered but that has control over dropped packets if there are any.
///
///  1. Reliable,
///  2. No guarantee for delivery
///  3. No guarantee that it is in order
///  4. Able to get dropped packets from the channel (udp with option to get dropped packets).
pub struct UnreliableChannel
{
    socket: UdpSocket,
    recv_buffer: [u8; UDP_BUFFER_SIZE],
}

impl UnreliableChannel
{
    pub fn new(socket: UdpSocket) -> Self
    {
        socket.set_nonblocking(true);
        UnreliableChannel { socket, recv_buffer: [0; UDP_BUFFER_SIZE] }
    }
}

impl Channel for UnreliableChannel
{
    fn send(&mut self, addr: SocketAddr, payload: &[u8]) -> io::Result<usize> {
        self.socket.send(payload)
    }

    fn recv(&mut self) ->  io::Result<Option<Packet>> {
        let (len, _addr) = self.socket.recv_from(&mut self.recv_buffer)?;

        if len > 0 {
            Ok(Some(Packet::new(_addr, self.recv_buffer[0..len].to_vec())))
        }else {
            Ok(None)
        }
    }
}