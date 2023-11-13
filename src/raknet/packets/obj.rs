use crate::raknet::objects::MsgBuffer;

pub trait FromBuffer {
    fn from_buffer(buf: &mut MsgBuffer) -> Self;
}

pub trait ToBuffer {
    fn to_buffer(&self) -> MsgBuffer;
}
