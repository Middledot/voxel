/// raknet/objects/reliability.rs
/// =============================
/// 
/// Class to hold reliability type and data.
/// Refer to frame.rs

use super::MsgBuffer;

pub enum ReliabilityType {
    Unreliable = 0,
    UnreliableSequenced,
    Reliable,
    ReliableOrdered,
    ReliableSequenced,
    UnreliableACK,
    ReliableACK,
    ReliableOrderedACK,
}

impl ReliabilityType {
    pub fn from_flags(flags: u8) -> Self {
        // stolen from NetrexMC
        // 10000000 & 11100000 = 10000000
        // 10000000 >> 5 = 00000100
        // 100 = 4?
        // vvv = 2???
        match (flags & 0b11100000) >> 5 {
            0 => ReliabilityType::Unreliable,
            1 => ReliabilityType::UnreliableSequenced,
            2 => ReliabilityType::Reliable,
            3 => ReliabilityType::ReliableOrdered,
            4 => ReliabilityType::ReliableSequenced,
            5 => ReliabilityType::UnreliableACK,
            6 => ReliabilityType::ReliableACK,
            7 => ReliabilityType::ReliableOrderedACK,
            _ => panic!("Uhm, excuse me"),
        }
    }
}

pub struct Reliability {
    _type: ReliabilityType,
    pub rel_frameindex: Option<u32>,
    pub seq_frameindex: Option<u32>,
    pub ord_frameindex: Option<u32>,
    pub ord_channel: Option<u8>,
}

impl Reliability {
    pub fn new(flags: u8) -> Self {
        Self {
            _type: ReliabilityType::from_flags(flags),
            rel_frameindex: None,
            seq_frameindex: None,
            ord_frameindex: None,
            ord_channel: None,
        }
    }

    pub fn extract(&mut self, buf: &mut MsgBuffer) {
        if self.is_reliable() {
            self.rel_frameindex = Some(buf.read_u24_le_bytes());
        }

        if self.is_sequenced() {
            self.seq_frameindex = Some(buf.read_u24_le_bytes());
        }

        if self.is_ordered() {
            self.ord_frameindex = Some(buf.read_u24_le_bytes());
            self.ord_channel = Some(buf.read_byte());
        }
    }

    // actually we could just use .unwrap() and hope for the best

    // pub fn get_rel_frameindex(&mut self) -> u32 {
    //     match self.rel_frameindex {
    //         Some(rel_frameindex) => rel_frameindex,
    //         None => panic!("Not sure what to do here")
    //     }
    // }

    // pub fn get_seq_frameindex(&mut self) -> u32 {
    //     match self.seq_frameindex {
    //         Some(seq_frameindex) => seq_frameindex,
    //         None => panic!("Not sure what to do here")
    //     }
    // }

    // pub fn get_ord_frameindex(&mut self) -> u32 {
    //     match self.ord_frameindex {
    //         Some(ord_frameindex) => ord_frameindex,
    //         None => panic!("Not sure what to do here")
    //     }
    // }

    // pub fn get_ord_channnel(&mut self) -> u8 {
    //     match self.ord_channel {
    //         Some(ord_channel) => ord_channel,
    //         None => panic!("Not sure what to do here")
    //     }
    // }

    pub fn get_type(&mut self) -> &ReliabilityType {
        &self._type
    }

    pub fn is_reliable(&mut self) -> bool {
        if let ReliabilityType::Reliable
        | ReliabilityType::ReliableSequenced
        | ReliabilityType::ReliableOrdered
        | ReliabilityType::ReliableACK
        | ReliabilityType::ReliableOrderedACK = self.get_type()
        {
            return true;
        }
        return false;
    }

    pub fn is_sequenced(&mut self) -> bool {
        if let ReliabilityType::UnreliableSequenced | ReliabilityType::ReliableSequenced =
            self.get_type()
        {
            return true;
        }
        return false;
    }

    pub fn is_ordered(&mut self) -> bool {
        // sequenced implies ordered
        if self.is_sequenced() {
            return true;
        }
        if let ReliabilityType::ReliableOrdered | ReliabilityType::ReliableOrderedACK =
            self.get_type()
        {
            return true;
        }
        return false;
    }
}
