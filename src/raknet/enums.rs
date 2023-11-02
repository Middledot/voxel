// this doesn't work
// TODO: remove?
// #[repr(u8)]
// pub enum PacketId {
//     OnlinePing = 0x00,
//     OfflinePing = 0x01,
//     OfflinePingOpenConn = 0x02,
//     OnlinePong = 0x03,
//     OfflinePong = 0x1c,
//     OppenConnRequest1 = 0x05,
//     OppenConnReply1 = 0x06,
//     OppenConnRequest2 = 0x07,
//     OppenConnReply2 = 0x08,
//     ConnRequest = 0x09,
//     ConnRequestAccepted = 0x10,
//     NewIncomingConn = 0x13,
//     Disconnect = 0x15,
//     IncompatibleProtocolVer = 0x19,
//     FrameSet0 = 0x80,
//     FrameSetF = 0x8d,
//     Nack = 0xa0,
//     Ack = 0xc0,
//     GamePacket = 0xfe,
// }

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

    // these are better than what I came up earlier so
    pub fn is_reliable(&self) -> bool {
        if let ReliabilityType::Reliable
        | ReliabilityType::ReliableSequenced
        | ReliabilityType::ReliableOrdered
        | ReliabilityType::ReliableACK
        | ReliabilityType::ReliableOrderedACK = self
        {
            return true;
        }
        return false;
    }

    pub fn is_sequenced(&self) -> bool {
        if let ReliabilityType::UnreliableSequenced | ReliabilityType::ReliableSequenced = self {
            return true;
        }
        return false;
    }

    pub fn is_ordered(&self) -> bool {
        if self.is_sequenced() {
            // sequenced implies ordered
            return true;
        }
        if let ReliabilityType::ReliableOrdered | ReliabilityType::ReliableOrderedACK = self {
            return true;
        }
        return false;
    }
}
