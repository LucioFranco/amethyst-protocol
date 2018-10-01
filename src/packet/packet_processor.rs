use std::io;

use packet::Packet;

pub struct PacketProcessor
{

}

impl PacketProcessor {
    pub fn process_data(data: &Vec<u8>) -> io::Result<Option<Packet>>
    {

    }
}