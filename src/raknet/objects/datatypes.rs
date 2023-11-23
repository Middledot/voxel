/// raknet/objects/datatypes.rs
/// ===========================
///
/// Functions for convert RakNet datatypes into rust and vice versa
/// Primarily used by MsgBuffer
/// Reference: https://wiki.vg/Raknet_Protocol#Data_types
///
/// TODO: cleanup and trimming (do we need all of these?)
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::{SystemTime, UNIX_EPOCH};

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
    newarr[..3].copy_from_slice(&bytes[..3]);

    u32::from_le_bytes(newarr)
}

pub fn to_u24_le_bytes(value: &u32) -> [u8; 3] {
    let result = value.to_le_bytes();
    let mut newarr = [0u8; 3];
    newarr[..3].copy_from_slice(&result[..3]);

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
#[allow(clippy::ptr_arg)]
pub fn from_address_bytes(version: u8, bytes: &Vec<u8>) -> SocketAddr {
    if version == 0x04 {
        SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3])),
            from_u16_be_bytes([bytes[4], bytes[5]]),
        )
    } else if version == 0x06 {
        // new stuff taken from nukkit
        // 0, 1 = family (0x17)
        // 2, 3 = port
        // 4, 5, 6, 7 = flow?
        // 8, 9, 10, 11, 12, 13, 14, 15 = address
        SocketAddr::new(
            IpAddr::V6(Ipv6Addr::new(
                bytes[8] as u16,
                bytes[9] as u16,
                bytes[10] as u16,
                bytes[11] as u16,
                bytes[12] as u16,
                bytes[13] as u16,
                bytes[14] as u16,
                bytes[15] as u16,
                // TODO: rewrite this
                // from_u16_be_bytes([bytes[12], bytes[13]]),
                // from_u16_be_bytes([bytes[14], bytes[15]]),
                // from_u16_be_bytes([bytes[16], bytes[17]]),
                // from_u16_be_bytes([bytes[18], bytes[19]]),
                // from_u16_be_bytes([bytes[20], bytes[21]]),
                // from_u16_be_bytes([bytes[22], bytes[23]]),
                // from_u16_be_bytes([bytes[24], bytes[25]]),
                // from_u16_be_bytes([bytes[26], bytes[27]]),
            )),
            from_u16_be_bytes([bytes[2], bytes[3]]),
        )
    } else {
        panic!("not supposed to happen?")
    }
}

pub fn to_address_bytes(addr: &SocketAddr) -> Vec<u8> {
    let mut address = vec![];

    if let IpAddr::V4(ip) = addr.ip() {
        address.push(0x04);
        address.extend_from_slice(&ip.octets());
        address.extend_from_slice(&to_u16_be_bytes(&addr.port()));
    } else if let IpAddr::V6(ip) = addr.ip() {
        address.push(0x06);
        address.extend_from_slice(&to_u16_be_bytes(&0x17));  // TODO: THIS IS LITTLE-ENDIAN
        address.extend_from_slice(&to_u16_be_bytes(&addr.port()));

        let octets = ip.octets();
        address.extend_from_slice(&[0, 0, octets[2], octets[3]]);
        for segment in ip.segments() {
            address.extend_from_slice(&segment.to_be_bytes());
        }
    }

    address
}

pub fn get_unix_milis() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Uhm... excuse me");
    since_the_epoch.as_millis()
}

// read_string

// pub fn write_string(value: &String, mut buffer: Vec<u8>) -> Vec<u8> {

// }
