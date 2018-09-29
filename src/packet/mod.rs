use net;
use std;
use std::io;

mod packet;
use packet::packet::Packet;


pub trait PacketSerialize
{
    fn serialize();
}

pub struct ReliablePacketSerialize
{
    packet: Packet,
}

impl PacketSerialize for ReliablePacketSerialize
{
    fn serialize() {
        unimplemented!()
    }
}

impl ReliablePacketSerialize
{
    pub fn new(packet: Packet) -> Self
    {
        ReliablePacketSerialize { packet }
    }
}

trait Channel
{
    fn send(&self, packet: Packet);
    fn recv(&self);
}

struct ReliableChannel
{
    socket: net::UdpSocket
}

impl Channel for ReliableChannel
{
    fn send(&self, packet: Packet) -> io::Result<usize> {
        self.socket.send(packet)
    }

    fn recv(&self) -> io::Result<Option<Packet>> {
        self.recv()
    }
}

struct UnreliableChannel
{
    socket: std::net::UdpSocket
}



