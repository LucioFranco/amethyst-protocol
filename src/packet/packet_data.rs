use super::header::{FragmentHeader, PacketHeader, HeaderParser};
use super::RawPacketData;

use std::io::{Result};

#[derive(Debug)]
/// Contains the raw data this packet exists of. Note that an packet can be divided into seperate fragments
pub struct PacketData
{
    parts: Vec<RawPacketData>
}

impl PacketData {
    pub fn new() -> PacketData
    {
        PacketData { parts: Vec::new() }
    }

    /// Add fragment to this packet
    pub fn add_fragment(&mut self, fragment: &HeaderParser<Output=Result<Vec<u8>>>, fragment_data: Vec<u8>)
    {
        self.parts.push(RawPacketData::new(fragment, fragment_data))
    }

    /// Return the total fragments this packet is divided into.
    pub fn fragment_count(&self) -> usize
    {
        return self.parts.len()
    }

    /// Return the parts this packet exists of.
    pub fn parts(&mut self) -> Vec<Vec<u8>>
    {
        let mut parts_data: Vec<Vec<u8>> = Vec::new();

        for part in self.parts.iter_mut() {
            parts_data.push(part.serialize());
        }

        parts_data
    }
}

