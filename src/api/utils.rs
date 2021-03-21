use std::convert::TryInto;
use num::{BigUint, Zero};
use std::ops::Shl;

/// Convert numbers in the Download Manifest from little indian and %03d concatenated string
pub fn blob_to_num(str: String) -> u128 {
    let mut num: u128 = 0;
    let mut shift: u128 = 0;
    for i in (0..str.len()).step_by(3) {
        if let Ok(n) = str[i..i + 3].parse::<u128>() {
            num += match n.checked_shl(shift as u32) {
                None => 0,
                Some(number) => number,
            };
            shift += 8;
        }
    }
    return num;
}

/// Convert BIG numbers in the Download Manifest from little indian and %03d concatenated string
pub fn bigblob_to_num(str: String) -> BigUint {
    let mut num: BigUint = BigUint::zero();
    let mut shift: u128 = 0;
    for i in (0..str.len()).step_by(3) {
        if let Ok(n) = str[i..i + 3].parse::<BigUint>() {
            num += n.shl(shift);
            shift += 8;
        }
    }
    return num;
}

pub(crate) fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
    let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
    matching == a.len() && matching == b.len()
}

pub(crate) fn read_le(buffer: &Vec<u8>, position: &mut usize) -> u32 {
    *position += 4;
    u32::from_le_bytes(buffer[*position - 4..*position].try_into().unwrap())
}

pub(crate) fn read_le_signed(buffer: &Vec<u8>, position: &mut usize) -> i32 {
    *position += 4;
    i32::from_le_bytes(buffer[*position - 4..*position].try_into().unwrap())
}

pub(crate) fn read_le_64(buffer: &Vec<u8>, position: &mut usize) -> u64 {
    *position += 8;
    u64::from_le_bytes(buffer[*position - 8..*position].try_into().unwrap())
}

pub(crate) fn read_le_64_signed(buffer: &Vec<u8>, position: &mut usize) -> i64 {
    *position += 8;
    i64::from_le_bytes(buffer[*position - 8..*position].try_into().unwrap())
}

pub(crate) fn read_fstring(buffer: &Vec<u8>, position: &mut usize) -> Option<String> {
    let mut length = read_le_signed(buffer, position);
    if length < 0 {
        length *= -2;
        *position += length as usize;
        Some(String::from_utf16_lossy(
            buffer[*position - length as usize..*position - 2]
                .chunks_exact(2)
                .into_iter()
                .map(|a| u16::from_ne_bytes([a[0], a[1]]))
                .collect::<Vec<u16>>()
                .as_slice(),
        ))
    } else if length > 0 {
        *position += length as usize;
        match std::str::from_utf8(&buffer[*position - length as usize..*position - 1]) {
            Ok(s) => Some(s.to_string()),
            Err(_) => None,
        }
    } else {
        None
    }
}