use std::default::Default;

pub struct NetworkConfig
{
    pub max_packet_size: usize,
    pub max_fragments: u32,
    pub fragment_size: usize,
    pub fragment_reassembly_buffer_size: usize,
    pub receive_buffer_max_size: usize,
}

impl Default for NetworkConfig{
    fn default() -> Self {
        Self {
            max_packet_size: 16 * 1024,
            max_fragments: 16,
            fragment_size: 1024,
            fragment_reassembly_buffer_size: 64,
            receive_buffer_max_size: 1500,
        }
    }
}