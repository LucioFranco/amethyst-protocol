pub struct FragmentPakcket
{
    crc32: u32,
    sequence: u16,
    packet_type: u8,
    fragment_id: u8,
    num_fragment: u8,


//[crc32] (32 bits)
//[sequence] (16 bits)
//[packet type = 0] (2 bits)
//[fragment id] (8 bits)
//[num fragments] (8 bits)
//[pad zero bits to nearest byte index]
//<fragment data>
}