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
            0 => ReliabilityType::Unreliable,          // 000
            1 => ReliabilityType::UnreliableSequenced, // 001
            2 => ReliabilityType::Reliable,            // 010
            3 => ReliabilityType::ReliableOrdered,     // 011
            4 => ReliabilityType::ReliableSequenced,   // 100
            5 => ReliabilityType::UnreliableACK,       // 101
            6 => ReliabilityType::ReliableACK,         // 110
            7 => ReliabilityType::ReliableOrderedACK,  // 111
            _ => panic!("Uhm, excuse me"),
        }
    }

    pub fn is_reliable(&self) -> bool {
        if let ReliabilityType::Reliable
        | ReliabilityType::ReliableSequenced
        | ReliabilityType::ReliableOrdered
        | ReliabilityType::ReliableACK
        | ReliabilityType::ReliableOrderedACK = self
        {
            return true;
        }
        false
    }

    pub fn is_sequenced(&self) -> bool {
        if self.is_ordered() {
            return true;
        }
        if let ReliabilityType::UnreliableSequenced | ReliabilityType::ReliableSequenced = self {
            return true;
        }
        false
    }

    pub fn is_ordered(&self) -> bool {
        // sequenced implies ordered
        if let ReliabilityType::ReliableOrdered | ReliabilityType::ReliableOrderedACK = self {
            return true;
        }
        false
    }
}

pub struct Reliability {
    reltype: ReliabilityType,
    pub rel_frameindex: Option<u32>,
    pub seq_frameindex: Option<u32>,
    pub ord_frameindex: Option<u32>,
    pub ord_channel: Option<u8>,
}

impl Reliability {
    // resource: http://www.jenkinssoftware.com/raknet/manual/reliabilitytypes.html
    pub fn extract(flags: u8, buf: &mut MsgBuffer) -> Self {
        let reltype = ReliabilityType::from_flags(flags);

        let mut rel_frameindex = None;
        let mut seq_frameindex = None;
        let mut ord_frameindex = None;
        let mut ord_channel = None;

        if reltype.is_reliable() {
            rel_frameindex = Some(buf.read_u24_le_bytes());
        }

        if reltype.is_sequenced() {
            seq_frameindex = Some(buf.read_u24_le_bytes());
        }

        if reltype.is_ordered() {
            ord_frameindex = Some(buf.read_u24_le_bytes());
            ord_channel = Some(buf.read_byte());
        }

        Self {
            reltype: ReliabilityType::from_flags(flags),
            rel_frameindex,
            seq_frameindex,
            ord_frameindex,
            ord_channel,
        }
    }

    pub fn is_reliable(&self) -> bool {
        self.reltype.is_reliable()
    }

    pub fn is_sequenced(&self) -> bool {
        self.reltype.is_sequenced()
    }

    pub fn is_ordered(&self) -> bool {
        // sequenced implies ordered
        self.reltype.is_ordered()
    }
}
