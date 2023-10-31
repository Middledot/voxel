// Functions to read and right RakNet datatypes to/from bytes

use std::io::Read;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};

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

    pub fn into_bytes(&mut self) -> &Vec<u8> {
        return &self.buffer;
    }

    pub fn len(&mut self) -> usize {
        return self.buffer.len();
    }

    pub fn rest_len(&mut self) -> usize {
        // possibly remove this??
        return self.buffer[self.pos..].len();
    }

    pub fn read(&mut self, num: u64, buf: &mut [u8]) -> usize {
        let res = self.buffer[self.pos..].take(num).read(buf);
        self.pos += num as usize;
        return res.expect("Failed to read");
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

    pub fn read_magic(&mut self) -> [u8; 16] {
        let mut magic = [0u8; 16];
        self.read(16, &mut magic);

        return magic;
    }

    pub fn write_magic(&mut self, magic: &[u8; 16]) {
        self.write(magic);
    }

    pub fn read_i16_be_bytes(&mut self) -> i16 {
        let mut result = [0u8; 2];
        self.read(8, &mut result);
    
        return from_i16_be_bytes(result);
    }
    
    pub fn write_i16_be_bytes(&mut self, value: &i16) {
        self.write(&to_i16_be_bytes(value));
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

// pub fn write_address(value: SocketAddr)

// read_string

// pub fn write_string(value: &String, mut buffer: Vec<u8>) -> Vec<u8> {

// }
