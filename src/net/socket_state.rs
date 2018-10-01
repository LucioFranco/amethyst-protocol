use std::collections::HashMap;
use bincode::{deserialize, serialize};

use super::{Connection,SocketAddr, NetworkConfig};
use packet::{header, Packet, PacketData};
use self::header::{FragmentHeader, PacketHeader};

/// This holds the 'virtual connections' currently (connected) to the udp socket.
pub struct SocketState {
    connections: HashMap<SocketAddr, Connection>,
}

impl SocketState {
    pub fn new() -> SocketState {
        SocketState {
            connections: HashMap::new(),
        }
    }

    /// This will initialize the seq number, ack number and if necessarily splits the packet up in different fragments.
    pub fn pre_process_packet(&mut self, packet: Packet, config: &NetworkConfig) -> (SocketAddr, PacketData) {
        let connection = self.create_connection_if_not_exists(&packet.addr);

        // queue new packet
        connection
            .waiting_packets
            .enqueue(connection.seq_num, packet.clone());

        let mut packet_data = PacketData::new();

        // create packet header
        let packet_header = PacketHeader::new(connection.seq_num, connection);

        let payload = packet.payload();
        let payload_length = payload.len();

        if payload_length <= config.fragment_size {
            // we don't need to splitup the packet packet
            packet_data.add_fragment(&packet_header, payload.to_vec());
        }else {
            // we need to split up the packet.
            // check how many fragments this packet should be seperated in.
            let remainder = if payload_length % config.fragment_size > 0 { 1 } else { 0 };
            let num_fragments = ((payload_length / config.fragment_size) + remainder) as usize;

            for fragment_id in 0..num_fragments{
                let fragment = FragmentHeader::new(fragment_id as u8, num_fragments as u8, packet_header.clone());

                // get start end pos in buffer
                let start_fragment_pos: usize = fragment_id * config.fragment_size;
                let mut end_fragment_pos: usize = (fragment_id + 1) * config.fragment_size;

                // If remaining buffer fits int one packet just set the end position to the length of the packet payload.
                if end_fragment_pos > (payload_length as usize) {
                    end_fragment_pos = (payload_length as usize);
                }

                // get specific slice of data for fragment
                let fragment_data = &payload[start_fragment_pos..end_fragment_pos];

                packet_data.add_fragment(&fragment, fragment_data.to_vec());

            }
        }

        // increase sequence number
        connection.seq_num = connection.seq_num.wrapping_add(1);

        (packet.addr, packet_data)
    }

    /// This will return all dropped packets from this connection.
    pub fn dropped_packets(&mut self, addr: SocketAddr) -> Vec<Packet> {
        let connection = self.create_connection_if_not_exists(&addr);
        connection.dropped_packets.drain(..).collect()
    }

    /// This will process an incoming packet and update acknowledgement information.
    pub fn process_received(&mut self, addr: SocketAddr, packet: &PacketHeader) {
        let mut connection = self.create_connection_if_not_exists(&addr);
        connection.their_acks.ack(packet.seq);

        // Update dropped packets if there are any.
        let dropped_packets = connection
            .waiting_packets
            .ack(packet.ack_seq, packet.ack_field);

        connection.dropped_packets = dropped_packets.into_iter().map(|(_, p)| p).collect();
    }

    #[inline]
    /// If there is no connection with the given socket address an new connection will be made.
    fn create_connection_if_not_exists(&mut self, addr: &SocketAddr) -> &mut Connection {
        self.connections.entry(*addr).or_insert(Connection::new())
    }
}
