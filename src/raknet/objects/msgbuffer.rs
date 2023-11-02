
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use super::datatypes::*;

pub struct MsgBuffer {
    buffer: Vec<u8>,
    pos: usize,
}

impl MsgBuffer {
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
        self.pos == self.buffer.len() - 1
    }

    pub fn into_bytes(&mut self) -> &Vec<u8> {
        &self.buffer
    }

    pub fn len(&mut self) -> usize {
        self.buffer.len()
    }

    pub fn len_rest(&mut self) -> usize {
        self.len() - self.pos
    }

    pub fn read(&mut self, num: u64, buf: &mut [u8]) -> usize {
        let res = self.buffer[self.pos..].take(num).read(buf);
        self.pos += num as usize;

        res.expect("Failed to read")
    }

    pub fn read_vec(&mut self, num: usize) -> Vec<u8> {
        let res = self.buffer[self.pos..self.pos + num].to_vec();
        self.pos += num;

        res
    }

    pub fn read_rest(&mut self) -> Vec<u8> {
        let res = self.buffer[self.pos..].to_vec();

        res
    }

    pub fn read_byte(&mut self) -> u8 {
        let result = self.buffer[self.pos];
        self.pos += 1;

        result
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

        from_i64_be_bytes(result)
    }

    pub fn write_i64_be_bytes(&mut self, value: &i64) {
        self.write(&to_i64_be_bytes(value));
    }

    pub fn read_i32_be_bytes(&mut self) -> i32 {
        let mut result = [0u8; 4];
        self.read(4, &mut result);

        from_i32_be_bytes(result)
    }

    pub fn write_i32_be_bytes(&mut self, value: &i32) {
        self.write(&to_i32_be_bytes(value));
    }

    pub fn read_u24_le_bytes(&mut self) -> u32 {
        // we pretend it's a u24 but really we're using u32
        let mut result = [0u8; 3];
        self.read(3, &mut result);

        from_u24_le_bytes_to_u32(result)
    }

    pub fn write_u24_le_bytes(&mut self, value: &u32) {
        self.write(&to_u24_le_bytes(value));
    }

    pub fn read_i16_be_bytes(&mut self) -> i16 {
        let mut result = [0u8; 2];
        self.read(8, &mut result);

        from_i16_be_bytes(result)
    }

    pub fn write_i16_be_bytes(&mut self, value: &i16) {
        self.write(&to_i16_be_bytes(value));
    }

    pub fn read_u16_be_bytes(&mut self) -> u16 {
        let mut result = [0u8; 2];
        self.read(2, &mut result);

        from_u16_be_bytes(result)
    }

    pub fn write_u16_be_bytes(&mut self, value: &u16) {
        self.write(&to_u16_be_bytes(value));
    }

    pub fn read_magic(&mut self) -> [u8; 16] {
        let mut magic = [0u8; 16];
        self.read(16, &mut magic);

        magic
    }

    pub fn write_magic(&mut self, magic: &[u8; 16]) {
        self.write(magic);
    }

    pub fn read_address(&mut self) -> SocketAddr {
        let _iptype = self.read_byte(); // TODO: support ipv6 https://wiki.vg/Raknet_Protocol#Data_types
        let address: SocketAddr;

        // if iptype == 0x04 {  // ipv4 vs ipv6
        let mut ip = [0u8; 6]; // 127 0 0 1
        self.read(6, &mut ip);
        address = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])),
            from_i16_be_bytes([ip[4], ip[5]]) as u16,
        ); // are you joking me right now (ports)

        address
    }

    pub fn write_address(&mut self, address: &SocketAddr) {
        // address.is_ipv4
        self.write_byte(0x04);
        let ip = match address.ip() {
            IpAddr::V4(addr) => addr.octets(),
            _ => panic!("uhm excuse me"),
        };

        self.write(&ip);
        self.write_i16_be_bytes(&(address.port() as i16));
    }
}