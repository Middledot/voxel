/// raknet/objects/frame.rs
/// =======================
///
/// Class that contains information of a frame, which is sent
/// in multiples with frame_set (packet ids 0x80 to 0x8d).
///
/// Reference: https://wiki.vg/Raknet_Protocol#Frame_Set_Packet
use super::FragmentInfo;
use super::MsgBuffer;
use super::Reliability;

pub struct Frame {
    pub flags: u8,
    pub bitlength: u16, // remove?
    pub bodysize: u16,
    pub reliability: Reliability,
    pub fragment_info: FragmentInfo,
    pub inner_packet_id: u8,
    pub body: MsgBuffer,
}

impl Frame {
    pub fn parse(buf: &mut MsgBuffer) -> Self {
        // so far, pretty much completely taken from PieMC
        let flags = buf.read_byte();
        let bitlength = buf.read_u16_be_bytes();

        let mut reliability = Reliability::new(flags);
        reliability.extract(buf);

        let mut fragment_info = FragmentInfo::new(flags);
        fragment_info.extract(buf);

        let bodysize = (bitlength + 7) / 8;
        println!("rel? {:?}", reliability.is_reliable());
        println!("seq? {:?}", reliability.is_sequenced());
        println!("ord? {:?}", reliability.is_ordered());

        // println!("{:?}", &flags);
        // println!("{:?}", &bitlength);
        // println!("{:?}", &bodysize);
        // println!("{:?}", &reliability.rel_frameindex.unwrap_or(234));
        // println!("{:?}", &reliability.seq_frameindex.unwrap_or(234));
        // println!("{:?}", &reliability.ord_frameindex.unwrap_or(234));
        // println!("{:?}", &reliability.ord_channel.unwrap_or(234));
        // println!("{:?}", &fragment_info.compound_size.unwrap_or(234));
        // println!("{:?}", &fragment_info.compound_id.unwrap_or(234));
        // println!("{:?}", &fragment_info.index.unwrap_or(234));
        let mut body = MsgBuffer::from(buf.read_vec(bodysize as usize));
        let inner_packet_id = body.read_byte();

        Self {
            flags,
            bitlength,
            bodysize,
            reliability,
            fragment_info,
            inner_packet_id,
            body,
        }
    }

    pub fn serialize(&mut self) -> MsgBuffer {
        let mut buf = MsgBuffer::new();
        buf.write_byte(self.flags);
        buf.write_u16_be_bytes(&self.bitlength);
        buf.write_u16_be_bytes(&self.bodysize);

        if self.reliability.is_reliable() {
            buf.write_u24_le_bytes(&self.reliability.rel_frameindex.unwrap())
        } else if self.reliability.is_sequenced() {
            buf.write_u24_le_bytes(&self.reliability.seq_frameindex.unwrap());
        } else {
            // ordered
            buf.write_u24_le_bytes(&self.reliability.ord_frameindex.unwrap());
            buf.write_byte(self.reliability.ord_channel.unwrap());
        }

        if self.fragment_info.is_fragmented {
            buf.write_i32_be_bytes(&self.fragment_info.compound_size.unwrap());
            buf.write_i16_be_bytes(&self.fragment_info.compound_id.unwrap());
            buf.write_i32_be_bytes(&self.fragment_info.index.unwrap());
        }

        buf.write_buffer(&mut self.body);

        buf
    }
}
