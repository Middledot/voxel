use super::MsgBuffer;
use super::Reliability;
use super::FragmentInfo;

pub struct Frame {
    flags: u8,
    bitlength: u16,  // remove?
    bytelength: u16,
    reliability: Reliability,
    fragment_info: FragmentInfo,
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

        let bytelength = (bitlength + 7) / 8;
        println!("rel? {:?}", reliability.is_reliable());
        println!("seq? {:?}", reliability.is_sequenced());
        println!("ord? {:?}", reliability.is_ordered());

        println!("{:?}", &flags);
        println!("{:?}", &bitlength);
        println!("{:?}", &bytelength);
        println!("{:?}", &reliability.rel_frameindex.unwrap_or(234));
        println!("{:?}", &reliability.seq_frameindex.unwrap_or(234));
        println!("{:?}", &reliability.ord_frameindex.unwrap_or(234));
        println!("{:?}", &reliability.ord_channel.unwrap_or(234));
        println!("{:?}", &fragment_info.compound_size.unwrap_or(234));
        println!("{:?}", &fragment_info.compound_id.unwrap_or(234));
        println!("{:?}", &fragment_info.index.unwrap_or(234));
        let body = buf.read_vec(bytelength as usize);
        println!("body: {:?}", &body);

        Self {
            flags,
            bitlength,
            bytelength,
            reliability,
            fragment_info,
        }
    }
}