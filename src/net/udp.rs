use bincode::{deserialize, serialize};
use packet::Packet;
use std::io;
use std::net::{self, ToSocketAddrs, SocketAddr};
use std::collections::HashMap;

const BUFFER_SIZE: usize = 1024;

pub struct UdpSocket {
    socket: net::UdpSocket,
    state: SocketState,
    recv_buffer: [u8; BUFFER_SIZE],
}

impl UdpSocket {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = net::UdpSocket::bind(addr)?;
        let state = SocketState::new();

        Ok(UdpSocket {
            socket,
            state,
            recv_buffer: [0; BUFFER_SIZE],
        })
    }

    pub fn recv(&mut self) -> io::Result<Option<Packet>> {
        // TODO: Pass addr back with packet
        let (len, _addr) = self.socket.recv_from(&mut self.recv_buffer)?;

        if len > 0 {
            // TODO: Remove unwrap and funnel result error typesq
            let packet: Packet = deserialize(&self.recv_buffer[..len]).unwrap();
            self.state.process_received(_addr, &packet);
            Ok(Some(packet))
        } else {
            Ok(None)
        }
    }

    pub fn send(&mut self, packet: &Packet) -> io::Result<usize> {
        let buf = serialize(packet).unwrap();
        let (addr, payload) = self.state.pre_process_packet(*packet);

        self.socket.send_to(&payload, addr)
    }

    pub fn set_nonblocking(&mut self, nonblocking: bool) -> io::Result<()> {
        self.socket.set_nonblocking(nonblocking)
    }
}

pub struct Connection
{
    pub seq_num: u16,
    pub dropped_packets: Vec<Packet>,
    pub waiting_packets: AckRecord,
    pub their_acks: ExternalAcks,
}

impl Connection {
    pub fn new() -> Connection {
        Connection {
            seq_num: 0,
            dropped_packets: Vec::new(),
            waiting_packets: AckRecord::new(),
            their_acks: ExternalAcks::new()
        }
    }
}

pub struct SocketState  {
    connections: HashMap<SocketAddr, Connection>
}

impl SocketState {
    pub fn new() -> SocketState {
        SocketState { connections: HashMap::new() }
    }

    pub fn pre_process_packet(&mut self, packet: Packet) -> (SocketAddr, Vec<u8>) {
        let connection = self.create_connection_if_not_exists(&addr);

        // queue new packet
        connection.waiting_packets.enqueue(connection.seq_num, packet.clone());

        // initialize packet data, seq, acked_seq etc.
        let final_packet = packet.with_data(connection.seq_num, connection.their_acks.last_seq, connection.their_acks.field);

        // increase sequence number
        connection.seq_num = connection.seq_num.wrapping_add(1);
        (packet.addr, final_packet.serialized())
    }

    pub fn dropped_packets(&mut self, addr: SocketAddr) -> Vec<Packet> {
        let connection = self.create_connection_if_not_exists(&addr);
        connection.dropped_packets.drain(..).collect()
    }

    pub fn process_received(&mut self, addr: SocketAddr, packet: &Packet) {
        let mut connection = self.create_connection_if_not_exists(&addr);
        connection.their_acks.ack(packet.seq);

        // get dropped packets
        let dropped_packets = connection.waiting_packets.ack(packet.ack_seq, packet.ack_field);
        connection.dropped_packets = dropped_packets.into_iter().map(|(_, p)| p).collect();
    }

    fn create_connection_if_not_exists(&mut self, add: &SocketAddr) -> &mut Connection
    {
        self.connections.entry(*addr).or_insert(Connection::new())
    }
}
