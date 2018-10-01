#[derive(Clone)]
/// This contains the information needed to know for reassembling fragments.
pub struct ReassemblyData {
    pub sequence: u16,
    pub num_fragments_received: usize,
    pub num_fragments_total: usize,
    pub buffer: Vec<u8>,
    pub fragments_received: [bool; 256]
}

impl ReassemblyData {
    pub fn new(sequence: u16, ack: u16, ack_bits: u32, num_fragments_total: usize, header_size: usize, prealloc: usize,) -> Self {
        Self {
            sequence,
            num_fragments_received: 0,
            num_fragments_total,
            buffer: Vec::with_capacity(prealloc),
            fragments_received: [false; 256],
        }
    }
}
impl Default for ReassemblyData {
    fn default() -> Self {
        Self {
            sequence: 0,
            num_fragments_received: 0,
            num_fragments_total: 0,
            buffer: Vec::with_capacity(1024),
            fragments_received: [false; 256]
        }
    }
}
