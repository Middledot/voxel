use super::{FromBuffer, ToBuffer};
use crate::raknet::objects::FragmentInfo;
use crate::raknet::objects::MsgBuffer;
use crate::raknet::objects::Reliability;
use crate::raknet::objects::datatypes::get_unix_milis;
use crate::raknet::objects::msgbuffer::PacketPriority;
use crate::raknet::objects::msgbuffer::SendPacket;
use crate::raknet::objects::reliability::ReliabilityType;

#[derive(Debug, Clone)]
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
    pub fn totalsize(&self) -> u16 {
        // 1 (flags)
        // 2 (bit length of body)
        // 3 (rel frame index) (if reliable)
        // 3 (seq frame index) (if sequenced)
        // 3 + 1 (ordered index + channel) (if ordered)
        // 4 + 2 + 4 (if fragmented)
        // + actual size of body (bytes)
        let mut size: u16 = 3;

        if self.reliability.is_reliable() {
            size += 3;
        }

        if self.reliability.is_sequenced() {
            size += 3;
        }

        if self.reliability.is_ordered() {
            size += 4;
        }

        if self.fragment_info.is_fragmented {
            size += 10;
        }

        size += self.bodysize;

        size
    }

    pub fn from_default_options(packet_id: u8, mut body: MsgBuffer, rel_frameindex: u32, ord_frameindex: u32) -> Self {
        Self {
            flags: 96,
            bitlength: (body.len() * 8) as u16,
            bodysize: body.len() as u16,
            reliability: Reliability {
                reltype: ReliabilityType::from_flags(96),
                rel_frameindex: Some(rel_frameindex),
                seq_frameindex: None,
                ord_frameindex: Some(ord_frameindex),
                ord_channel: Some(0)
            },
            fragment_info: FragmentInfo { is_fragmented: false, compound_size: None, compound_id: None, index: None },
            inner_packet_id: packet_id,
            body,
        }
    }

    // pub fn from_old_frame(frame: &Frame) -> Self {

    // }
}

impl FromBuffer for Frame {
    fn from_buffer(buf: &mut MsgBuffer) -> Self {
        // so far, pretty much completely taken from PieMC
        let flags = buf.read_byte();
        let bitlength = buf.read_u16_be_bytes();

        let reliability = Reliability::extract(flags, buf);

        let mut fragment_info = FragmentInfo::new(flags);
        fragment_info.extract(buf);

        let bodysize = (bitlength + 7) / 8;
        // println!("rel? {:?}", reliability.is_reliable());
        // println!("seq? {:?}", reliability.is_sequenced());
        // println!("ord? {:?}", reliability.is_ordered());

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
}

impl ToBuffer for Frame {
    fn to_buffer(&self) -> MsgBuffer {
        let mut buf = MsgBuffer::new();
        buf.write_byte(self.flags);
        buf.write_u16_be_bytes(self.bitlength);
        // buf.write_u16_be_bytes(&self.bodysize);

        if self.reliability.is_reliable() {
            buf.write_u24_le_bytes(self.reliability.rel_frameindex.unwrap())
        }

        if self.reliability.is_sequenced() {
            buf.write_u24_le_bytes(self.reliability.seq_frameindex.unwrap());
        }

        if self.reliability.is_ordered() {
            // ordered
            buf.write_u24_le_bytes(self.reliability.ord_frameindex.unwrap());
            buf.write_byte(self.reliability.ord_channel.unwrap());
        }

        if self.fragment_info.is_fragmented {
            buf.write_i32_be_bytes(self.fragment_info.compound_size.unwrap());
            buf.write_i16_be_bytes(self.fragment_info.compound_id.unwrap());
            buf.write_i32_be_bytes(self.fragment_info.index.unwrap());
        }

        buf.write_byte(self.inner_packet_id);
        buf.write_buffer(self.body.get_bytes());

        buf  // 3794229544342144685
    }
}


// #[derive(Debug)]
// enum FrameSetFlags {
/*
FLAG_VALID = 0b10000000;
FLAG_ACK = 0b01000000;
FLAG_HAS_B_AND_AS = 0b00100000;
FLAG_NACK = 0b00100000;
FLAG_PACKET_PAIR = 0b00010000;
FLAG_CONTINUOUS_SEND = 0b00001000;
FLAG_NEEDS_B_AND_AS = 0b00000100;
*/


#[derive(Debug)]
pub struct FrameSet {
    // pub flags: u8,
    pub index: u32,
    pub frames: Vec<Frame>,
}

impl FrameSet {
    pub fn currentsize(&self) -> u16 {
        self.frames.iter().map(|f| f.totalsize()).sum::<u16>() + 4
    }

    pub fn add_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    pub fn package(&self, priority: PacketPriority) -> SendPacket {
        SendPacket {
            packet_id: 0x80,
            body: self.to_buffer(),
            priority
        }
    }

    // pub fn try_add_frame(&mut self, frame: Frame, mtu: u16) -> Option<FrameSet> {
    //     if self.currentsize() + frame.totalsize() > mtu {
    //         let mut new_frameset = FrameSet {index: self.index+1, frames: vec![frame]};
    //         return Some(new_frameset)
    //     }

    // }
}

impl FromBuffer for FrameSet {
    fn from_buffer(/*flags: u8, */buf: &mut MsgBuffer) -> Self {
        let index = buf.read_u24_le_bytes();
        let mut frames: Vec<Frame> = vec![];

        while !buf.at_end() {
            frames.push(Frame::from_buffer(buf))
        }

        Self { /*flags,*/ index, frames }
    }
}

impl ToBuffer for FrameSet {
    fn to_buffer(&self) -> MsgBuffer {
        let mut buf = MsgBuffer::new();
        // buf.write_byte();
        buf.write_u24_le_bytes(self.index);

        for fr in self.frames.iter() {
            buf.write_buffer(fr.to_buffer().get_bytes());
        }

        buf
    }
}
