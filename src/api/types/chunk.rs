use std::convert::TryInto;
use log::{debug, error, warn};
use flate2::read::ZlibDecoder;
use std::io::Read;

/// Struct holding data for downloaded chunks
pub struct Chunk {}

impl Chunk {
    /// Parse chunk from binary vector
    pub fn from_vec(mut buffer: Vec<u8>) -> Option<Chunk> {
        let mut res = Chunk {};

        let mut position: usize = 0;
        let magic = crate::api::utils::read_le(&buffer, &mut position);
        if magic != 2986228386 {
            error!("No header magic");
            return None;
        }
        let header_version = crate::api::utils::read_le(&buffer, &mut position);
        println!("header_version: {}", header_version);
        let mut header_size = crate::api::utils::read_le(&buffer, &mut position);
        println!("header_size: {}", header_size);
        let mut compressed_size = crate::api::utils::read_le(&buffer, &mut position);
        println!("compressed_size: {}", compressed_size);
        let mut guid = format!(
            "{:08x}{:08x}{:08x}{:08x}",
            crate::api::utils::read_le(&buffer, &mut position),
            crate::api::utils::read_le(&buffer, &mut position),
            crate::api::utils::read_le(&buffer, &mut position),
            crate::api::utils::read_le(&buffer, &mut position)
        );
        println!("guid: {}", guid);
        let mut hash = crate::api::utils::read_le_64(&buffer, &mut position);
        println!("hash: {}", hash);
        let stored_as = buffer[position];
        position += 1;

        println!("stored_as: {}", stored_as);
        if header_version >= 2 {
            position += 20;
            let sha_hash: Vec<u8> = buffer[position - 20..position].try_into().unwrap();
            println!(
                "sha_hash: {}",
                sha_hash
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>()
            );
            let hash_type = buffer[position];
            position += 1;
            println!("hash_type: {}", hash_type);
        }
        if header_version >= 3 {
            let uncompressed_size = crate::api::utils::read_le(&buffer, &mut position);
            println!("uncompressed_size: {}", uncompressed_size);
        }
        let data = if stored_as == 1 {
            let mut z = ZlibDecoder::new(&buffer[position..]);
            let mut data: Vec<u8> = Vec::new();
            z.read_to_end(&mut data).unwrap();
            data
        } else {
            buffer[position..]
        };
        None
    }
}
