/// datatypes.rs
/// ============
/// 
/// Functions for convert RakNet datatypes into rust and vice versa
/// Primarily used by MsgBuffer
/// 
/// TODO: cleanup and trimming (do we need all of these?)

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

pub fn from_i64_be_bytes(bytes: [u8; 8]) -> i64 {
    i64::from_be_bytes(bytes)
}

pub fn to_i64_be_bytes(value: &i64) -> [u8; 8] {
    value.to_be_bytes()
}

pub fn from_i32_be_bytes(bytes: [u8; 4]) -> i32 {
    i32::from_be_bytes(bytes)
}

pub fn to_i32_be_bytes(value: &i32) -> [u8; 4] {
    value.to_be_bytes()
}

pub fn from_u24_le_bytes_to_u32(bytes: [u8; 3]) -> u32 {
    let mut newarr = [0u8; 4];
    for i in 0..3 {  // why
        newarr[i + 1] = bytes[i];
    }

    u32::from_le_bytes(newarr)
}

pub fn to_u24_le_bytes(value: &u32) -> [u8; 3] {
    let result = value.to_le_bytes();
    let mut newarr = [0u8; 3];
    for i in 0..3 {
        newarr[i] = result[i + 1];
    }

    newarr
}

pub fn from_i16_be_bytes(bytes: [u8; 2]) -> i16 {
    i16::from_be_bytes(bytes)
}

pub fn to_i16_be_bytes(value: &i16) -> [u8; 2] {
    value.to_be_bytes()
}

pub fn from_u16_be_bytes(bytes: [u8; 2]) -> u16 {
    u16::from_be_bytes(bytes)
}

pub fn to_u16_be_bytes(value: &u16) -> [u8; 2] {
    value.to_be_bytes()
}

// TODO: test this (especially ipv6)
pub fn from_address_bytes(version: u8, bytes: &Vec<u8>) -> SocketAddr {
    let address: SocketAddr;
    if version == 0x04 {
        address = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3])),
            from_u16_be_bytes([bytes[4], bytes[5]]),
        );
    } else {  // ver == 0x06
        address = SocketAddr::new(
            IpAddr::V6(Ipv6Addr::new(
                // TODO: rewrite this
                from_u16_be_bytes([bytes[12], bytes[13]]),
                from_u16_be_bytes([bytes[14], bytes[15]]),
                from_u16_be_bytes([bytes[16], bytes[17]]),
                from_u16_be_bytes([bytes[18], bytes[19]]),
                from_u16_be_bytes([bytes[20], bytes[21]]),
                from_u16_be_bytes([bytes[22], bytes[23]]),
                from_u16_be_bytes([bytes[24], bytes[25]]),
                from_u16_be_bytes([bytes[26], bytes[27]])
            )),
            from_u16_be_bytes([bytes[1], bytes[2]]),
        );
    }

    address
}

pub fn to_address_bytes(addr: &SocketAddr) -> Vec<u8> {
    let mut address = vec![];

    if let IpAddr::V4(ip) = addr.ip() {
        address.push(0x04);
        address.extend_from_slice(&ip.octets());
        address.extend_from_slice(&to_u16_be_bytes(&addr.port()));
    } else if let IpAddr::V6(ip) = addr.ip() {
        address.push(0x06);
        address.extend_from_slice(&to_u16_be_bytes(&0x17));
        address.extend_from_slice(&to_u16_be_bytes(&addr.port()));
        
        let octets = ip.octets();
        address.extend_from_slice(&[0, 0, octets[2], octets[3]]);
        for segment in ip.segments() {
            address.extend_from_slice(&segment.to_be_bytes());
        }
    }

    address
}

// read_string

// pub fn write_string(value: &String, mut buffer: Vec<u8>) -> Vec<u8> {

// }
