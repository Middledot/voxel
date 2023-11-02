use super::msgbuffer::MsgBuffer;

pub struct FragmentInfo {
    pub is_fragmented: bool,
    pub compound_size: Option<i32>,  // TODO: rename to just size?
    pub compound_id: Option<i16>,
    pub index: Option<i32>
}

impl FragmentInfo {
    pub fn new(flags: u8) -> Self {
        Self {
            is_fragmented: (flags & 1) != 0,
            compound_size: None,
            compound_id: None,
            index: None,
        }
    }

    pub fn extract(&mut self, buf: &mut MsgBuffer) {
        if self.is_fragmented {
            self.compound_size = Some(buf.read_i32_be_bytes());
            self.compound_id = Some(buf.read_i16_be_bytes());
            self.index = Some(buf.read_i32_be_bytes());
        }
    }
}
