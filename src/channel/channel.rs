use std::io;
use std::net::{SocketAddr, UdpSocket as UnreliableUdp};

use super::{Channel, ChannelType, ReliableUnorderedChannel, UnreliableChannel};
use packet::Packet;
use net::UdpSocket as ReliableUdp;

type ChannelImpl = Box<Channel + Send + Sync>;

pub struct CommunicationChannel
{
    channel: ChannelIml
}

impl CommunicationChannel
{
    /// Create new Communication Channel from the given ChannelType and sets the listener on the given host.
    pub fn new(channel_type: ChannelType, host: SocketAddr) -> Self
    {
        let channel: ChannelIml;

        match channel_type
        {
            ChannelType::ReliableUnordered =>
            {
                let udp_socket = ReliableUdp::bind(host).unwrap();
                channel = Box::from(ReliableUnorderedChannel::new(udp_socket)) as ChannelIml
            },
            ChannelType::Unreliable =>
            {
                let udp_socket = UnreliableUdp::bind(host).unwrap();
                channel = Box::from(UnreliableChannel::new(udp_socket)) as ChannelIml;
            },
        };

        CommunicationChannel { channel }
    }

    /// Send information to the given endpoint.
    pub fn send(&mut self, addr: SocketAddr, payload: &[u8])  -> io::Result<usize> {
        self.channel.send(addr, payload)
    }

    /// Receive data from the channel and return the packet if there is result.
    pub fn recv(&mut self) -> io::Result<Option<Packet>>{
        self.channel.recv()
    }
}

// TODO: These tests may fail but if runned seperately they succeed. Maybe because of receive can not receive because there is not data jet to read.
// TODO: So we need to implement something to make channel able to be a blocking channel or non blocking.
mod tests {
    use std::net::{SocketAddr, IpAddr};
    use std::str::{self, FromStr};
    use std::io;

    use packet::Packet;
    use channel::{CommunicationChannel, ChannelType};

    #[test]
    fn send_packet_reliable_unordered_channel()
    {
        let ip_sender = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12345);
        let ip_receiver = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12346);

        let mut channel_sender = CommunicationChannel::new(ChannelType::ReliableUnordered, ip_sender);
        let mut channel_receiver = CommunicationChannel::new(ChannelType::ReliableUnordered, ip_receiver);

        let packet = channel_receiver.recv();
        channel_sender.send(ip_receiver, &[123]);

        let parsed = packet.unwrap().unwrap();

        assert_eq!(parsed.payload(), &[123]);
    }

    #[test]
    fn send_packet_unreliable_channel()
    {
        use std::{thread, time};

        let ip_sender = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12345);
        let ip_receiver = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12346);

        let mut channel_sender = CommunicationChannel::new(ChannelType::Unreliable, ip_sender);
        let mut channel_receiver = CommunicationChannel::new(ChannelType::Unreliable, ip_receiver);

        thread::spawn(move || {
            thread::sleep(time::Duration::from_millis(20));
            let packet: Result<Option<Packet>, io::Error> = channel_receiver.recv();

            let parsed: Packet = packet.unwrap().unwrap();

            assert_eq!(parsed.payload(), &[123]);
        });

        channel_sender.send(ip_receiver, &[123]);
    }
}