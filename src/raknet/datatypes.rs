// Functions to read and right RakNet datatypes to/from bytes

use std::io::Read;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};

use super::enums::ReliabilityType;

pub struct MsgBuffer {
    buffer: Vec<u8>,
    pos: usize,
}

impl MsgBuffer {
    // maybe reconsider naming lol
    pub fn new() -> Self {
        Self {
            buffer: vec![],
            pos: 0,
        }
    }

    pub fn from(buffer: Vec<u8>) -> Self {
        Self {
            buffer: buffer,
            pos: 0,
        }
    }

    pub fn at_end(&mut self) -> bool {
        return self.pos == self.buffer.len()-1;
    }

    pub fn into_bytes(&mut self) -> &Vec<u8> {
        return &self.buffer;
    }

    pub fn len(&mut self) -> usize {
        return self.buffer.len();
    }

    pub fn len_rest(&mut self) -> usize {
        return self.len()-self.pos;
    }

    pub fn read(&mut self, num: u64, buf: &mut [u8]) -> usize {
        println!("ff{:?}", &self.pos);
        println!("ff{:?}", &num);
        println!("ff{:?}", num as usize+self.pos);
        let res = self.buffer[self.pos..].take(num).read(buf);
        self.pos += num as usize;
        return res.expect("Failed to read");
    }

    pub fn read_vec(&mut self, num: usize) -> Vec<u8> {
        println!("ee{:?}", &num);
        println!("ee{:?}", &self.pos);
        println!("ee{:?}", self.buffer.len()-1);
        let res = self.buffer[self.pos..self.pos+num].to_vec();
        self.pos += num;
        return res;
    }

    pub fn read_rest(&mut self) -> Vec<u8> {
        let res = self.buffer[self.pos..].to_vec();
        // self.pos += num as usize;
        return res;
    }

    pub fn read_byte(&mut self) -> u8 {
        let result = self.buffer[self.pos];
        self.pos += 1;
        return result;
    }

    pub fn write(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn write_byte(&mut self, data: u8) {
        self.buffer.push(data);
    }

    pub fn read_i64_be_bytes(&mut self) -> i64 {
        let mut result = [0u8; 8];
        self.read(8, &mut result);

        return from_i64_be_bytes(result);
    }

    pub fn write_i64_be_bytes(&mut self, value: &i64) {
        self.write(&to_i64_be_bytes(value));
    }

    pub fn read_i32_be_bytes(&mut self) -> i32 {
        let mut result = [0u8; 4];
        self.read(4, &mut result);

        return from_i32_be_bytes(result);
    }

    pub fn write_i32_be_bytes(&mut self, value: &i32) {
        self.write(&to_i32_be_bytes(value));
    }

    pub fn read_u24_le_bytes(&mut self) -> u32 {
        // we pretend it's a u24 but really we're using u32
        let mut result = [0u8; 3];
        self.read(3, &mut result);
    
        return from_u24_le_bytes_to_u32(result);
    }

    pub fn write_u24_le_bytes(&mut self, value: &u32) {
        self.write(&to_u24_le_bytes(value));
    }

    pub fn read_i16_be_bytes(&mut self) -> i16 {
        let mut result = [0u8; 2];
        self.read(8, &mut result);
    
        return from_i16_be_bytes(result);
    }

    pub fn write_i16_be_bytes(&mut self, value: &i16) {
        self.write(&to_i16_be_bytes(value));
    }

    pub fn read_u16_be_bytes(&mut self) -> u16 {
        let mut result = [0u8; 2];
        self.read(2, &mut result);
    
        return from_u16_be_bytes(result);
    }

    pub fn write_u16_be_bytes(&mut self, value: &u16) {
        self.write(&to_u16_be_bytes(value));
    }

    pub fn read_magic(&mut self) -> [u8; 16] {
        let mut magic = [0u8; 16];
        self.read(16, &mut magic);

        return magic;
    }

    pub fn write_magic(&mut self, magic: &[u8; 16]) {
        self.write(magic);
    }

    pub fn read_address(&mut self) -> SocketAddr {
        let _iptype = self.read_byte();  // TODO: support ipv6 https://wiki.vg/Raknet_Protocol#Data_types
        let address: SocketAddr;
    
        // if iptype == 0x04 {  // ipv4 vs ipv6
        let mut ip = [0u8; 6];  // 127 0 0 1
        self.read(6, &mut ip);
        address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])), from_i16_be_bytes([ip[4], ip[5]]) as u16);  // are you joking me right now (ports)

        return address;
    }

    pub fn write_address(&mut self, address: &SocketAddr) {
        // address.is_ipv4
        self.write_byte(0x04);
        let ip = match address.ip() {
            IpAddr::V4(addr) => addr.octets(),
            _ => panic!("uhm excuse me")
        };

        self.write(&ip);
        self.write_i16_be_bytes(&(address.port() as i16));
    }
}


pub struct Frame {

}

impl Frame {
    pub fn parse(buf: &mut MsgBuffer) -> Self {
        // so far, pretty much completely taken from PieMC
        let flags = buf.read_byte();
        let bitlength = buf.read_u16_be_bytes();

        let reliability = ReliabilityType::from_flags(flags);
        let rreliability = ReliabilityType::from_flags(flags);
        println!("{:?}", rreliability as u8);
        let fragmented = (flags & 1) != 0;

        let mut rel_frameindex: u32 = 234;
        let mut seq_frameindex: u32 = 234;
        let mut ord_frameindex: u32 = 234;
        let mut ord_chnl: u8 = 234;

        let mut compound_size: i32 = 234;
        let mut compound_id: i16 = 234;
        let mut index: i32 = 234;

        if reliability.is_reliable() {
            rel_frameindex = buf.read_u24_le_bytes();
        }

        if reliability.is_sequenced() {
            seq_frameindex = buf.read_u24_le_bytes();
        }

        if reliability.is_ordered() {
            ord_frameindex = buf.read_u24_le_bytes();
            ord_chnl = buf.read_byte();
        }

        if fragmented {
            compound_size = buf.read_i32_be_bytes();
            compound_id = buf.read_i16_be_bytes();
            index = buf.read_i32_be_bytes();
        }

        let bytesize = (bitlength + 7) / 8;
        
        println!("{:?}", &flags);
        println!("{:?}", &bitlength);
        println!("{:?}", &bytesize);
        println!("{:?}", &rel_frameindex);
        println!("{:?}", &seq_frameindex);
        println!("{:?}", &ord_frameindex);
        println!("{:?}", &ord_chnl);
        println!("{:?}", &compound_size);
        println!("{:?}", &compound_id);
        println!("{:?}", &index);
        let body = buf.read_vec(bytesize as usize);
        println!("body: {:?}", &body);

        Self {}
    }
}

pub fn from_i64_be_bytes(bytes: [u8; 8]) -> i64 {
    return i64::from_be_bytes(bytes);
}

pub fn to_i64_be_bytes(value: &i64) -> [u8; 8] {
    return value.to_be_bytes();
}

pub fn from_i16_be_bytes(bytes: [u8; 2]) -> i16 {
    return i16::from_be_bytes(bytes);
}

pub fn to_i16_be_bytes(value: &i16) -> [u8; 2] {
    return value.to_be_bytes();
}

pub fn from_u16_be_bytes(bytes: [u8; 2]) -> u16 {
    return u16::from_be_bytes(bytes);
}

pub fn to_u16_be_bytes(value: &u16) -> [u8; 2] {
    return value.to_be_bytes();
}

pub fn from_u24_le_bytes_to_u32(bytes: [u8; 3]) -> u32 {
    let mut newarr = [0u8; 4];
    for i in 0..3 {
        newarr[i + 1] = bytes[i];  // why
    };

    return u32::from_le_bytes(newarr);
}

pub fn to_u24_le_bytes(value: &u32) -> [u8; 3] {
    let result = value.to_le_bytes();
    let mut newarr = [0u8; 3];
    for i in 0..3 {
        newarr[i] = result[i+1];
    }

    return newarr;
}

pub fn from_i32_be_bytes(bytes: [u8; 4]) -> i32 {
    return i32::from_be_bytes(bytes);
}

pub fn to_i32_be_bytes(value: &i32) -> [u8; 4] {
    return value.to_be_bytes();
}

// pub fn write_address(value: SocketAddr)

// read_string

// pub fn write_string(value: &String, mut buffer: Vec<u8>) -> Vec<u8> {

// }
