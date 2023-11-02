//! Functions that convert the datatypes used by the Bedrock client

pub fn from_i64_be_bytes(bytes: [u8; 8]) -> i64 {
    i64::from_be_bytes(bytes)
}

pub fn to_i64_be_bytes(value: &i64) -> [u8; 8] {
    value.to_be_bytes()
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

pub fn from_i32_be_bytes(bytes: [u8; 4]) -> i32 {
    i32::from_be_bytes(bytes)
}

pub fn to_i32_be_bytes(value: &i32) -> [u8; 4] {
    value.to_be_bytes()
}

// pub fn write_address(value: SocketAddr)

// read_string

// pub fn write_string(value: &String, mut buffer: Vec<u8>) -> Vec<u8> {

// }
