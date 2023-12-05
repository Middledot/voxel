/// raknet/objects/msgbuffer.rs
/// ===========================
///
/// A wrapper class to make it easier to read and
/// write bytes.
use std::io::Read;
use std::net::SocketAddr;

use super::datatypes::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum PacketPriority {
    Immediate = 8,
    High = 4,
    Medium = 2,
    Low = 1,
}

#[derive(Debug)]
pub struct Packet {
    pub packet_id: u8,
    // TODO: if this eventually has a use, change all the time I just
    // plugged in an outdated timestamp
    pub timestamp: u128,
    pub body: MsgBuffer,
}

#[derive(Clone, Eq, PartialEq)]
pub struct SendPacket {
    pub packet_id: u8,
    pub body: MsgBuffer,
    pub priority: PacketPriority,
}

impl Ord for SendPacket {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for SendPacket {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
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
        Self { buffer, pos: 0 }
    }

    pub fn at_end(&mut self) -> bool {
        self.pos == self.buffer.len()
    }

    pub fn get_bytes(&self) -> &Vec<u8> {
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

    pub fn write_i64_be_bytes(&mut self, value: i64) {
        self.write(&to_i64_be_bytes(value));
    }

    pub fn read_i32_be_bytes(&mut self) -> i32 {
        let mut result = [0u8; 4];
        self.read(4, &mut result);

        from_i32_be_bytes(result)
    }

    pub fn write_i32_be_bytes(&mut self, value: i32) {
        self.write(&to_i32_be_bytes(value));
    }

    pub fn read_f32_le_bytes(&mut self) -> f32 {
        let mut result = [0u8; 4];
        self.read(4, &mut result);

        from_f32_le_bytes(result)
    }

    pub fn write_f32_le_bytes(&mut self, value: f32) {
        self.write(&to_f32_le_bytes(value));
    }

    pub fn read_u24_le_bytes(&mut self) -> u32 {
        // we pretend it's a u24 but really we're using u32
        let mut result = [0u8; 3];
        self.read(3, &mut result);

        from_u24_le_bytes_to_u32(result)
    }

    pub fn write_u24_le_bytes(&mut self, value: u32) {
        self.write(&to_u24_le_bytes(value));
    }

    pub fn read_i16_be_bytes(&mut self) -> i16 {
        let mut result = [0u8; 2];
        self.read(2, &mut result);

        from_i16_be_bytes(result)
    }

    pub fn write_i16_be_bytes(&mut self, value: i16) {
        self.write(&to_i16_be_bytes(value));
    }

    pub fn read_u16_be_bytes(&mut self) -> u16 {
        let mut result = [0u8; 2];
        self.read(2, &mut result);

        from_u16_be_bytes(result)
    }

    pub fn write_u16_be_bytes(&mut self, value: u16) {
        self.write(&to_u16_be_bytes(value));
    }

    pub fn read_u16_le_bytes(&mut self) -> u16 {
        let mut result = [0u8; 2];
        self.read(2, &mut result);

        from_u16_le_bytes(result)
    }

    pub fn write_u16_le_bytes(&mut self, value: u16) {
        self.write(&to_u16_le_bytes(value));
    }

    pub fn read_magic(&mut self) -> [u8; 16] {
        let mut magic = [0u8; 16];
        self.read(16, &mut magic);

        magic
    }

    pub fn write_magic(&mut self, magic: &[u8; 16]) {
        self.write(magic);
    }

    pub fn write_string(&mut self, str: &String) {
        let str: Vec<u8> = str.as_bytes().to_vec();
        let str_len = (str.len()) as i16;

        self.write_i16_be_bytes(str_len);
        self.write(&str);
    }

    pub fn read_address(&mut self) -> SocketAddr {
        let ipver = self.read_byte();

        if ipver == 0x04 {
            let mut bytes = [0u8; 6]; // 7-1
            self.read(6, &mut bytes);
            from_address_bytes(ipver, &bytes.to_vec())
        } else {
            // if ipver == 0x06
            // new changes from nukkit
            let mut bytes = [0u8; 28]; // 29-1
            self.read(28, &mut bytes);
            from_address_bytes(ipver, &bytes.to_vec())
        }
    }

    pub fn write_address(&mut self, address: &SocketAddr) {
        self.write(&to_address_bytes(address));
    }

    pub fn write_buffer(&mut self, other: &[u8]) {
        self.buffer.extend_from_slice(other)
    }

    pub fn read_i32_varint_bytes(&mut self) -> i32 {
        from_i32_varint_bytes(self)
    }

    pub fn read_zigzag32(&mut self) -> i32 {
        // https://gist.github.com/mfuerstenau/ba870a29e16536fdbaba
        // https://lemire.me/blog/2022/11/25/making-all-your-integers-positive-with-zigzag-encoding/
        let val = self.read_i32_be_bytes();
        if val < 0 {
            return -2 * val - 1;
        }
        2 * val
    }

    // implement other zigzags
}
