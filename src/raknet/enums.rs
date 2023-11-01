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
    Unreliable,
    UnreliableSequenced,
    Reliable,
    ReliableSequenced,
    ReliableOrdered,
    UnreliableWithACK,
    ReliableWithACK,
    ReliableOrderedWithACK
}
