use num::{BigUint, Zero};
use std::convert::TryInto;
use std::ops::Shl;
use std::borrow::BorrowMut;
use std::num::ParseIntError;
use std::cmp::Ordering;

/// Convert numbers in the Download Manifest from little indian and %03d concatenated string
pub fn blob_to_num<T: Into<String>>(str: T) -> u128 {
    let mut num: u128 = 0;
    let mut shift: u128 = 0;
    let string = str.into();
    for i in (0..string.len()).step_by(3) {
        if let Ok(n) = string[i..i + 3].parse::<u128>() {
            num += n.checked_shl(shift as u32).unwrap_or(0);
            shift += 8;
        }
    }
    num
}

/// Convert BIG numbers in the Download Manifest from little indian and %03d concatenated string
pub fn bigblob_to_num<T: Into<String>>(str: T) -> BigUint {
    let mut num: BigUint = BigUint::zero();
    let mut shift: u128 = 0;
    let string = str.into();
    for i in (0..string.len()).step_by(3) {
        if let Ok(n) = string[i..i + 3].parse::<BigUint>() {
            num += n.shl(shift);
            shift += 8;
        }
    }
    num
}

pub(crate) fn do_vecs_match<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
    matching == a.len() && matching == b.len()
}

pub(crate) fn read_le(buffer: &[u8], position: &mut usize) -> u32 {
    *position += 4;
    u32::from_le_bytes(buffer[*position - 4..*position].try_into().unwrap())
}

pub(crate) fn read_le_signed(buffer: &[u8], position: &mut usize) -> i32 {
    *position += 4;
    i32::from_le_bytes(buffer[*position - 4..*position].try_into().unwrap())
}

pub(crate) fn read_le_64(buffer: &[u8], position: &mut usize) -> u64 {
    *position += 8;
    u64::from_le_bytes(buffer[*position - 8..*position].try_into().unwrap())
}

pub(crate) fn read_le_64_signed(buffer: &[u8], position: &mut usize) -> i64 {
    *position += 8;
    i64::from_le_bytes(buffer[*position - 8..*position].try_into().unwrap())
}

pub(crate) fn read_fstring(buffer: &[u8], position: &mut usize) -> Option<String> {
    let mut length = read_le_signed(buffer, position);
    match length.cmp(&0) {
        Ordering::Less => {
            length *= -2;
            *position += length as usize;
            Some(String::from_utf16_lossy(
                buffer[*position - length as usize..*position - 2]
                    .chunks_exact(2)
                    .map(|a| u16::from_ne_bytes([a[0], a[1]]))
                    .collect::<Vec<u16>>()
                    .as_slice(),
            ))
        }
        Ordering::Equal => { None }
        Ordering::Greater => {
            *position += length as usize;
            match std::str::from_utf8(&buffer[*position - length as usize..*position - 1]) {
                Ok(s) => Some(s.to_string()),
                Err(_) => None,
            }
        }
    }
}

pub(crate) fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub(crate) fn write_fstring(string: String) -> Vec<u8> {
    let mut meta: Vec<u8> = Vec::new();
    if !string.is_empty() {
        meta.append(
            ((string.len() + 1) as u32)
                .to_le_bytes()
                .to_vec()
                .borrow_mut(),
        );
        meta.append(string.into_bytes().borrow_mut());
        meta.push(0);
    } else {
        meta.append(0u32.to_le_bytes().to_vec().borrow_mut())
    }
    meta
}

#[cfg(test)]
mod tests {
    use crate::api::utils::{
        bigblob_to_num, blob_to_num, do_vecs_match, read_fstring, read_le, read_le_64,
        read_le_64_signed, read_le_signed,
    };
    use num::bigint::ToBigUint;

    #[test]
    fn vector_match() {
        let a = vec![0, 0, 0];
        let b = vec![0, 0, 0];
        assert_eq!(do_vecs_match(&a, &b), true);
    }

    #[test]
    fn vector_not_match() {
        let a = vec![0, 0, 0];
        let b = vec![0, 0, 1];
        assert_eq!(do_vecs_match(&a, &b), false);
    }

    #[test]
    fn blob_to_num_test() {
        assert_eq!(blob_to_num("165045004000"), 273829)
    }

    #[test]
    fn blob_to_bignum_test() {
        assert_eq!(
            bigblob_to_num("165045004000"),
            ToBigUint::to_biguint(&273829).unwrap()
        )
    }

    #[test]
    fn read_le_test() {
        let mut position: usize = 0;
        let buffer = vec![1, 2, 3, 4];
        assert_eq!(read_le(&buffer, &mut position), 67305985);
        assert_eq!(position, 4)
    }

    #[test]
    fn read_le_signed_test() {
        let mut position: usize = 0;
        let buffer = vec![237, 201, 255, 255];
        assert_eq!(read_le_signed(&buffer, &mut position), -13843);
        assert_eq!(position, 4)
    }

    #[test]
    fn read_le_64_test() {
        let mut position: usize = 0;
        let buffer = vec![0, 0, 5, 3, 0, 1, 2, 3];
        assert_eq!(read_le_64(&buffer, &mut position), 216736831629492224);
        assert_eq!(position, 8)
    }

    #[test]
    fn read_le_64_signed_test() {
        let mut position: usize = 0;
        let buffer = vec![237, 201, 255, 255, 255, 255, 255, 255];
        assert_eq!(read_le_64_signed(&buffer, &mut position), -13843);
        assert_eq!(position, 8)
    }

    #[test]
    fn read_fstring_utf8() {
        let mut position: usize = 0;
        let buffer = vec![5, 0, 0, 0, 97, 98, 99, 100, 0];
        assert_eq!(
            read_fstring(&buffer, &mut position),
            Some("abcd".to_string())
        );
        assert_eq!(position, 9)
    }

    #[test]
    fn read_fstring_utf16() {
        let mut position: usize = 0;
        let buffer = vec![251, 255, 255, 255, 97, 0, 98, 0, 99, 0, 100, 0, 0, 0];
        assert_eq!(
            read_fstring(&buffer, &mut position),
            Some("abcd".to_string())
        );
        assert_eq!(position, 14)
    }
}
