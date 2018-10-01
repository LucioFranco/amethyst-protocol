use std::collections::HashMap;
use std::io::{self, Cursor, ErrorKind, Error, Read, Write};
use std::net::{self, SocketAddr, ToSocketAddrs};

use super::{SocketState, NetworkConfig};
use packet::{header, Packet, SequenceBuffer, ReassemblyData};
use self::header::{FragmentHeader, PacketHeader, HeaderParser, HeaderReader};

use bincode::{deserialize, serialize};

pub struct UdpSocket {
    socket: net::UdpSocket,
    state: SocketState,
    recv_buffer: [u8;1452],
    config: NetworkConfig,
    reassembly_buffer: SequenceBuffer<ReassemblyData>
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
            reassembly_buffer: SequenceBuffer::with_capacity(config.fragment_reassembly_buffer_size),
            config: config,
        })
    }

    pub fn recv(&mut self) -> io::Result<Option<Packet>> {
        let (len, _addr) = { self.socket.recv_from(&mut self.recv_buffer)? };

        if len > 0 {
            let packet = self.recv_buffer[..len].to_owned();
            let prefix_byte = packet[0];
            let mut cursor = Cursor::new(packet);

            let mut received_bytes = Ok(None);

            if prefix_byte & 1 == 0 {
               received_bytes = self.handle_normal_packet(&mut cursor, &_addr);
            } else {
               received_bytes = self.handle_fragment(&mut cursor);
            }

            match received_bytes {
                Ok(Some(payload)) => return Ok(Some(Packet::new(_addr, payload))),
                Ok(None) => return Ok(None),
                Err(e) => return Err(e)
            };
        } else {
            return Ok(None)
        }
    }

    pub fn send(&mut self, mut packet: Packet) -> io::Result<usize> {
        let (addr, mut packet_data) = self.state.pre_process_packet(packet, &self.config);

        let mut bytes_send: usize = 0;

        for payload in packet_data.parts() {
            let result: io::Result<usize> = self.socket.send_to(&payload, addr);


            match result {
                Ok(len) => {
                    bytes_send += len;
                },
                Err(e) => return Err(e)
            }
        }

        Ok(bytes_send)
    }

    pub fn set_nonblocking(&mut self, nonblocking: bool) -> io::Result<()> {
        self.socket.set_nonblocking(nonblocking)
    }

    fn handle_fragment(&mut self, cursor: &mut Cursor<Vec<u8>>) -> io::Result<Option<Vec<u8>>>
    {
        // read fragment packet
        let fragment_header = FragmentHeader::read(cursor)?;

        // if fragment does not exists we need to insert a new entry
        if !self.reassembly_buffer.exists(fragment_header.sequence) {
            if fragment_header.id == 0 {
                if fragment_header.packet_header.is_none() {
                    return Err(Error::new(ErrorKind::Other, "Invalid Fragment"));
                }

                let packet_header = fragment_header.packet_header.unwrap();
                let ack = packet_header.ack_seq;
                let ack_bits = packet_header.ack_field;

                let reassembly_data = ReassemblyData::new(fragment_header.sequence, ack, ack_bits, usize::from(fragment_header.num_fragments), fragment_header.size() as usize, 9 + self.config.fragment_size);

                self.reassembly_buffer.insert(reassembly_data.clone(), fragment_header.sequence);
            } else {
                return Err(Error::new(ErrorKind::Other, "Invalid Fragment"));
            }
        }

        let mut num_fragments_received = 0;
        let mut num_fragments_total = 0;
        let mut sequence = 0;
        let mut total_buffer = Vec::new();

        {
            // get entry of previous received fragments
            let reassembly_data = match self.reassembly_buffer.get_mut(fragment_header.sequence) {
                Some(val) => val,
                None => return Err(Error::new(ErrorKind::Other, "Invalid Fragment"))
            };

            // Got the data
            if reassembly_data.num_fragments_total != usize::from(fragment_header.num_fragments) {
                return Err(Error::new(ErrorKind::Other, "Invalid Fragment"));
            }

            if reassembly_data.fragments_received[usize::from(fragment_header.id)] {
                return Err(Error::new(ErrorKind::Other, "Invalid Fragment"));
            }

            // increase number of received fragments and set the specific fragment to received.
            reassembly_data.num_fragments_received += 1;
            reassembly_data.fragments_received[usize::from(fragment_header.id)] = true;

            // read payload after fragment header
            let mut payload = Vec::new();
            cursor.read_to_end(&mut payload)?;

            // add the payload from the fragment to the buffer whe have in cache
            reassembly_data.buffer.write(payload.as_slice());

            num_fragments_received = reassembly_data.num_fragments_received;
            num_fragments_total = reassembly_data.num_fragments_total;
            sequence = reassembly_data.sequence as u16;
            total_buffer = reassembly_data.buffer.clone();
        }

        // if whe received all fragments then remove entry and return the total received bytes.
        if num_fragments_received == num_fragments_total {
            let sequence = sequence as u16;
            self.reassembly_buffer.remove(sequence);

            return Ok(Some(total_buffer))
        }

        return Ok(None);
    }

    fn handle_normal_packet(&mut self, cursor: &mut Cursor<Vec<u8>>, addr: &SocketAddr) -> io::Result<Option<Vec<u8>>>
    {
        let packet_header = PacketHeader::read(cursor);

        match packet_header {
            Ok(header) => {
                self.state.process_received(*addr, &header);

                let mut payload = Vec::new();
                cursor.read_to_end(&mut payload)?;

                Ok(Some(payload))
            }
            Err(e) => { return Err(e) }
        }
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
    fn send_receive_1_pckt() {
        let mut send_socket = UdpSocket::bind("127.0.0.1:12347").unwrap();
        let mut recv_socket = UdpSocket::bind("127.0.0.1:12348").unwrap();

        let addr = SocketAddr::new(
            IpAddr::from_str("127.0.0.1").expect("Unreadable input IP."),
            12348,
        );

        let dummy_packet = Packet::new(addr, vec![1, 2, 3]);

        let send_result: io::Result<usize> = send_socket.send(dummy_packet);
        println!("{:?}", send_result);
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
                        println!("lenght: {:?}", packet.payload().len());
                        break;
                    }
                    _ => {}

                };
            }
        });

        let dummy_packet = Packet::new(addr, vec![123;4000]);
        let send_result: io::Result<usize> = send_socket.send(dummy_packet);
        assert!(send_result.is_ok());

        handle.join();
    }

    #[test]
    pub fn send_receive_stress_test() {
        const TOTAL_PACKAGES: u16 = 10000;

        thread::spawn(|| {
            thread::sleep(time::Duration::from_millis(3));

            let mut send_socket = UdpSocket::bind("127.0.0.1:12345").unwrap();

            let addr = SocketAddr::new(
                IpAddr::from_str("127.0.0.1").expect("Unreadable input IP."),
                12346,
            );

            for packet_count in 0..TOTAL_PACKAGES {
                let stub = StubData {
                    id: packet_count,
                    b: 1,
                };

                let data = serialize(&stub).unwrap();
                let len = data.len();

                let dummy_packet = Packet::new(addr, data);

                let send_result: io::Result<usize> = send_socket.send(dummy_packet);

                assert!(send_result.is_ok());
//                 println!(
//                     "sending packet_count: {} packet_id: {}",
//                     packet_count, stub.id
//                 );
                assert_eq!(send_result.unwrap(), 12);
            }
        });

        thread::spawn(|| {
            let mut recv_socket = UdpSocket::bind("127.0.0.1:12346").unwrap();

            let mut received_packages_count = 0;

            loop {
                let packet: io::Result<Option<Packet>> = recv_socket.recv();

                assert!(packet.is_ok());

                let packet_payload: Option<Packet> = packet.unwrap();
                assert!(packet_payload.is_some());
                let received_packet = packet_payload.unwrap();

                let stub_data = deserialize::<StubData>(received_packet.payload()).unwrap();

                assert_eq!(received_packet.addr().to_string(), "127.0.0.1:12345");
                assert_eq!(stub_data.id, received_packages_count);
                assert_eq!(stub_data.b, 1);

//                 println!(
//                     "receiving packet_count: {} packet_id: {}",
//                     received_packages_count, stub_data.id
//                 );

                received_packages_count += 1;

                if received_packages_count == TOTAL_PACKAGES {
                    break;
                }
            }
        }).join();
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
