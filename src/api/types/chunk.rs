use flate2::read::ZlibDecoder;
use log::{debug, error};
use std::convert::TryInto;
use std::io::Read;

/// Struct holding data for downloaded chunks
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Chunk {
    header_version: u32,
    header_size: u32,
    compressed_size: u32,
    guid: String,
    hash: u64,
    compressed: bool,
    sha_hash: Option<Vec<u8>>,
    /// 1 = rolling hash, 2 = sha hash, 3 = both
    hash_type: Option<u8>,
    uncompressed_size: Option<u32>,
    data: Vec<u8>,
}

impl Chunk {
    /// Parse chunk from binary vector
    pub fn from_vec(buffer: Vec<u8>) -> Option<Chunk> {
        let mut position: usize = 0;
        let magic = crate::api::utils::read_le(&buffer, &mut position);
        if magic != 2986228386 {
            error!("No header magic");
            return None;
        }
        let mut res = Chunk {
            header_version: crate::api::utils::read_le(&buffer, &mut position),
            header_size: crate::api::utils::read_le(&buffer, &mut position),
            compressed_size: crate::api::utils::read_le(&buffer, &mut position),
            guid: format!(
                "{:08x}{:08x}{:08x}{:08x}",
                crate::api::utils::read_le(&buffer, &mut position),
                crate::api::utils::read_le(&buffer, &mut position),
                crate::api::utils::read_le(&buffer, &mut position),
                crate::api::utils::read_le(&buffer, &mut position)
            ),
            hash: crate::api::utils::read_le_64(&buffer, &mut position),
            compressed: match buffer[position] {
                0 => false,
                _ => true,
            },
            sha_hash: None,
            hash_type: None,
            uncompressed_size: None,
            data: vec![],
        };
        position += 1;

        if res.header_version >= 2 {
            position += 20;
            res.sha_hash = Some(buffer[position - 20..position].try_into().unwrap());
            res.hash_type = Some(buffer[position]);
            position += 1;
        }
        if res.header_version >= 3 {
            res.uncompressed_size = Some(crate::api::utils::read_le(&buffer, &mut position));
        }
        debug!("Got chunk: {:?}", res);
        res.data = if res.compressed {
            let mut z = ZlibDecoder::new(&buffer[position..]);
            let mut data: Vec<u8> = Vec::new();
            z.read_to_end(&mut data).unwrap();
            data
        } else {
            buffer[position..].to_vec()
        };
        Some(res)
    }
}
