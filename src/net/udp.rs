use std::io::{self, Cursor, ErrorKind, Error, Read, Write};
use std::net::{self, SocketAddr, ToSocketAddrs};

use super::{SocketState, NetworkConfig};
use packet::{header, Packet, PacketProcessor};
use error::{NetworkError, Result};

pub struct UdpSocket {
    socket: net::UdpSocket,
    state: SocketState,
    recv_buffer: [u8;1452],
    config: NetworkConfig,
    packet_processor: PacketProcessor
}

impl UdpSocket {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = net::UdpSocket::bind(addr)?;
        let state = SocketState::new();
        // TODO: add functionality to get config from file.
        let config = NetworkConfig::default();

        Ok(UdpSocket {
            socket,
            state,
            recv_buffer: [0;1452],
            packet_processor: PacketProcessor::new(config.clone()),
            config: config,
        })
    }

    pub fn recv(&mut self) -> io::Result<Option<Packet>> {
        let (len, _addr) = { self.socket.recv_from(&mut self.recv_buffer)? };

        if len > 0 {
            let packet = self.recv_buffer[..len].to_owned();

            self.packet_processor.process_data(packet, _addr, &mut self.state)
        }else {
            return Err (Error::new(ErrorKind::Other, "Received bytes where of length 0"));
        }
    }

    pub fn send(&mut self, mut packet: Packet) -> Result<usize> {
        let (addr, mut packet_data) = self.state.pre_process_packet(packet, &self.config)?;

        let mut bytes_send: usize = 0;

        for payload in packet_data.parts() {
            let result: io::Result<usize> = self.socket.send_to(&payload, addr);

            match result {
                Ok(len) => {
                    bytes_send += len;
                },
                Err(e) => return Err(NetworkError::SendFailed.into())
            }
        }

        Ok(bytes_send)
    }

    pub fn set_nonblocking(&mut self, nonblocking: bool) -> io::Result<()> {
        self.socket.set_nonblocking(nonblocking)
    }
}

#[cfg(test)]
mod test {
    use super::UdpSocket;
    use bincode::{deserialize, serialize};
    use packet::Packet;
    use std::io;
    use std::net::{IpAddr, SocketAddr};
    use std::str::FromStr;
    use std::{thread, time};

    #[test]
    #[ignore]
    fn send_receive_1_pckt() {
        let mut send_socket = UdpSocket::bind("127.0.0.1:12347").unwrap();
        let mut recv_socket = UdpSocket::bind("127.0.0.1:12348").unwrap();

        let addr = SocketAddr::new(
            IpAddr::from_str("127.0.0.1").expect("Unreadable input IP."),
            12348,
        );

        let dummy_packet = Packet::new(addr, vec![1, 2, 3]);

        let send_result = send_socket.send(dummy_packet);
        assert!(send_result.is_ok());

        let packet: io::Result<Option<Packet>> = recv_socket.recv();
        assert!(packet.is_ok());
        let packet_payload: Option<Packet> = packet.unwrap();
        assert!(packet_payload.is_some());
        let received_packet = packet_payload.unwrap();

        assert_eq!(received_packet.addr().to_string(), "127.0.0.1:12347");
        assert_eq!(received_packet.payload(), &[1, 2, 3]);
    }

    #[test]
    fn send_receive_fragment_packet() {
        let mut send_socket = UdpSocket::bind("127.0.0.1:12347").unwrap();
        let mut recv_socket = UdpSocket::bind("127.0.0.1:12348").unwrap();

        let addr = SocketAddr::new(
            IpAddr::from_str("127.0.0.1").expect("Unreadable input IP."),
            12348,
        );

        let handle = thread::spawn(move || {
            loop {
                let packet: io::Result<Option<Packet>> = recv_socket.recv();

                match packet {
                    Ok(Some(packet)) => {
                        assert_eq!(packet.addr().to_string(), "127.0.0.1:12347");
                        assert_eq!(packet.payload(), vec![123; 4000].as_slice());
                        break;
                    }
                    Err(e) => { panic!(); },
                    _ => { }
                };
            }
        });

        let dummy_packet = Packet::new(addr, vec![123;4000]);
        let send_result = send_socket.send(dummy_packet);
        assert!(send_result.is_ok());

        handle.join();
    }

    #[ignore]
    pub fn send_receive_stress_test() {
        const TOTAL_PACKAGES: u16 = 1000;

        thread::spawn(|| {
            thread::sleep(time::Duration::from_millis(3));

            let mut send_socket = UdpSocket::bind("127.0.0.1:12357").unwrap();

            let addr = SocketAddr::new(
                IpAddr::from_str("127.0.0.1").expect("Unreadable input IP."),
                12358,
            );

            for packet_count in 0..TOTAL_PACKAGES {
                let stub = StubData {
                    id: packet_count,
                    b: 1,
                };

                let data = serialize(&stub).unwrap();
                let len = data.len();

                let dummy_packet = Packet::new(addr, data);
                let send_result = send_socket.send(dummy_packet);
                assert!(send_result.is_ok());
                assert_eq!(send_result.unwrap(), 12);
            }
        });

        thread::spawn(|| {
            let mut recv_socket = UdpSocket::bind("127.0.0.1:12358").unwrap();

            let mut received_packages_count = 0;

            loop {
                let packet: io::Result<Option<Packet>> = recv_socket.recv();

                assert!(packet.is_ok());

                let packet_payload: Option<Packet> = packet.unwrap();
                assert!(packet_payload.is_some());
                let received_packet = packet_payload.unwrap();

                let stub_data = deserialize::<StubData>(received_packet.payload()).unwrap();

                assert_eq!(received_packet.addr().to_string(), "127.0.0.1:12357");
                assert_eq!(stub_data.id, received_packages_count);
                assert_eq!(stub_data.b, 1);

                received_packages_count += 1;
                if received_packages_count >= TOTAL_PACKAGES {
                    break;
                }
            }
        }).join()
        .unwrap();
    }

    #[derive(Serialize, Deserialize, Clone, Copy)]
    struct StubData {
        pub id: u16,
        pub b: u16,
    }

    pub fn dummy_packet() -> Packet {
        let addr = SocketAddr::new(
            IpAddr::from_str("0.0.0.0").expect("Unreadable input IP."),
            12345,
        );

        Packet::new(addr, Vec::new())
    }
}
