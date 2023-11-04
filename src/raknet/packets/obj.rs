use crate::raknet::objects::MsgBuffer;

pub trait Deserialise {
    const ID: u8;

    fn deserialise(buf: &mut MsgBuffer) -> Self;
}

pub trait Serialize {
    const ID: u8;

    fn serialize(&self) -> MsgBuffer;
}
