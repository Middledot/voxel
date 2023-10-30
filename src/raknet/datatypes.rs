// Functions to read and right RakNet datatypes to/from bytes

use std::io::Read;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};

pub fn read_i64_be_bytes(buffer: Vec<u8>) -> (Vec<u8>, i64) {
    let mut result = [0u8; 8];
    buffer.take(8).read(&mut result).expect("Damn");
    
    return (buffer[8..].to_vec(), i64::from_be_bytes(result));
}

pub fn write_i64_be_bytes(value: &i64, mut buffer: Vec<u8>) -> Vec<u8> {
    let bytes: [u8; 8] = value.to_be_bytes();

    buffer.extend_from_slice(&bytes);
    return buffer;
}

pub fn read_magic(buffer: Vec<u8>) -> (Vec<u8>, [u8; 16]) {
    let mut result = [0u8; 16];
    buffer.take(16).read(&mut result).expect("Damn");
    
    // Unpack the bytes into an i64 value
    return (buffer[16..].to_vec(), result);
}

pub fn write_magic(value: &[u8], mut buffer: Vec<u8>) -> Vec<u8> {
    buffer.extend_from_slice(&value);
    return buffer;
}

pub fn read_i16_be_bytes(buffer: Vec<u8>) -> (Vec<u8>, i16) {
    let mut result = [0u8; 2];
    buffer.take(8).read(&mut result).expect("Damn");

    return (buffer[2..].to_vec(), i16::from_be_bytes(result));
}

pub fn write_i16_be_bytes(value: &i16, mut buffer: Vec<u8>) -> Vec<u8> {
    let bytes: [u8; 2] = value.to_be_bytes();

    buffer.extend_from_slice(&bytes);
    return buffer;
}

pub fn read_address(buffer: Vec<u8>) -> (Vec<u8>, SocketAddr) {
    let iptype = buffer[0];
    let address: SocketAddr;

    // if iptype == 0x04 {  // ipv4 vs ipv6
    let ip = buffer[1..=4].to_vec();  // 127 0 0 1
    address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])), buffer[5] as u16);

    return (buffer[7..].to_vec(), address);
}

// pub fn write_address(value: SocketAddr)

// read_string

// pub fn write_string(value: &String, mut buffer: Vec<u8>) -> Vec<u8> {

// }
