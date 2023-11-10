use crate::raknet::objects::MsgBuffer;

pub trait Deserialise {
    fn deserialise(buf: &mut MsgBuffer) -> Self;
}

pub trait Serialize {
    fn serialize(&self) -> MsgBuffer;
}
