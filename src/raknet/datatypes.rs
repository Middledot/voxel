// Functions to read and right RakNet datatypes to/from bytes

use std::io::Read;

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

// read_i16_be_bytes

pub fn write_i16_be_bytes(value: &i16, mut buffer: Vec<u8>) -> Vec<u8> {
    let bytes: [u8; 2] = value.to_be_bytes();

    buffer.extend_from_slice(&bytes);
    return buffer;
}

// read_string

// pub fn write_string(value: &String, mut buffer: Vec<u8>) -> Vec<u8> {

// }
