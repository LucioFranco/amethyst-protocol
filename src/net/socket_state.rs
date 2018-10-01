use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use std::io;

use bincode::{deserialize, serialize};

use packet::{header, Packet, PacketData};
use super::{Connection,SocketAddr, NetworkConfig};
use self::header::{FragmentHeader, PacketHeader};

use error::{NetworkError, Result};

// Type aliases
// Number of seconds we will wait until we consider a Connection to have timed out
type ConnectionTimeout = u64;
type ConnectionMap = Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<Connection>>>>>;

// Default timeout of 10 seconds
const TIMEOUT_DEFAULT: ConnectionTimeout = 10;

// Default time between checks of all clients for timeouts in seconds
const TIMEOUT_POLL_INTERVAL: u64 = 1;

/// This holds the 'virtual connections' currently (connected) to the udp socket.
pub struct SocketState {
    timeout: ConnectionTimeout,
    connections: ConnectionMap,
}

impl SocketState {
    pub fn new() -> SocketState {
        let mut socket_state = SocketState {
            connections: Arc::new(RwLock::new(HashMap::new())),
            timeout: TIMEOUT_DEFAULT,
        };
        socket_state.check_for_timeouts();
        socket_state
    }

    pub fn with_client_timeout(mut self, timeout: ConnectionTimeout) -> SocketState {
        self.timeout = timeout;
        self
    }

    /// This will initialize the seq number, ack number and give back the raw data of the packet with the updated information.
    pub fn pre_process_packet(&mut self, packet: Packet, config: &NetworkConfig) ->  Result<(SocketAddr, PacketData)>  {
        let connection = self.create_connection_if_not_exists(&packet.addr)?;

        let mut lock = connection
            .write()
            .map_err(|_| NetworkError::AddConnectionToManagerFailed)?;

        let connection_seq = lock.seq_num;
        // queue new packet
        lock.waiting_packets.enqueue(connection_seq, packet.clone());

        let mut packet_data = PacketData::new();

        // create packet header
        let packet_header = PacketHeader::new(connection_seq, lock.their_acks.last_seq, lock.their_acks.field);


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

        lock.seq_num = lock.seq_num.wrapping_add(1);
        Ok((packet.addr, packet_data))
    }

    /// This will return all dropped packets from this connection.
    pub fn dropped_packets(&mut self, addr: SocketAddr) -> Result<Vec<Packet>> {
        let connection = self.create_connection_if_not_exists(&addr)?;
        let mut lock = connection
            .write()
            .map_err(|_| NetworkError::AddConnectionToManagerFailed)?;

        let packets = lock.dropped_packets.drain(..).collect();
        Ok(packets)
    }

    /// This will process an incoming packet and update acknowledgement information.
    pub fn process_received(&mut self, addr: SocketAddr, packet: &PacketHeader) -> Result<()>{
        let connection = self.create_connection_if_not_exists(&addr)?;
        let mut lock = connection
            .write()
            .map_err(|_| NetworkError::AddConnectionToManagerFailed)?;

        lock.their_acks.ack(packet.seq);

        // Update dropped packets if there are any.
        let dropped_packets = lock
            .waiting_packets
            .ack(packet.ack_seq, packet.ack_field);

        Ok(())
    }

    // Regularly checks the last_heard attribute of all the connections in the manager to see if any have timed out
    fn check_for_timeouts(&mut self) {
        let connections_lock = self.connections.clone();
        let sleepy_time = Duration::from_secs(self.timeout);
        let poll_interval = Duration::from_secs(TIMEOUT_POLL_INTERVAL);

        thread::Builder::new()
            .name("check_for_timeouts".into())
            .spawn(move || loop {
                let connections = connections_lock.read().expect("Unable to aquire read lock");

                for (key, value) in connections.iter() {
                    let connection = value.read().expect("Unable to aquire read lock");
                    let last_heard = connection.last_heard();
                    if last_heard >= sleepy_time {
                        // TODO: pass up client TimedOut event
                        error!("Client has timed out: {:?}", key);
                    }
                }

                thread::sleep(poll_interval)
            }).unwrap();
    }

    #[inline]
    /// If there is no connection with the given socket address an new connection will be made.
    fn create_connection_if_not_exists(
        &mut self,
        addr: &SocketAddr,
    ) -> Result<Arc<RwLock<Connection>>> {
        let mut lock = self
            .connections
            .write()
            .map_err(|_| NetworkError::AddConnectionToManagerFailed)?;

        let connection = lock
            .entry(*addr)
            .or_insert_with(|| Arc::new(RwLock::new(Connection::new(*addr))));

        Ok(connection.clone())
    }
}

#[cfg(test)]
mod test {
    use super::SocketState;
    use net::connection::Connection;
    use std::net::ToSocketAddrs;
    use std::{thread, time};
    static TEST_HOST_IP: &'static str = "127.0.0.1";
    static TEST_BAD_HOST_IP: &'static str = "800.0.0.1";
    static TEST_PORT: &'static str = "20000";

    #[test]
    fn test_create_connection() {
        let addr = format!("{}:{}", TEST_HOST_IP, TEST_PORT).to_socket_addrs();
        assert!(addr.is_ok());
        let mut addr = addr.unwrap();
        let new_conn = Connection::new(addr.next().unwrap());
    }

    #[test]
    fn test_invalid_addr_fails() {
        let addr = format!("{}:{}", TEST_BAD_HOST_IP, TEST_PORT).to_socket_addrs();
        assert!(addr.is_err());
    }

    #[test]
    fn test_poll_for_invalid_clients() {
        let mut socket_state = SocketState::new();
        socket_state.check_for_timeouts();
        thread::sleep(time::Duration::from_millis(10000));
    }
}
