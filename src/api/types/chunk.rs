use flate2::read::ZlibDecoder;
use log::{debug, error};
use std::io::Read;

/// Struct holding data for downloaded chunks
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Chunk {
    header_version: u32,
    header_size: u32,
    compressed_size: u32,
    /// Guid of the chunk
    pub guid: String,
    /// Chunk Hash
    pub hash: u64,
    compressed: bool,
    /// Chunk sha hash
    pub sha_hash: Option<Vec<u8>>,
    /// 1 = rolling hash, 2 = sha hash, 3 = both
    pub hash_type: Option<u8>,
    /// Total chunk size
    pub uncompressed_size: Option<u32>,
    /// Chunk data
    pub data: Vec<u8>,
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
            compressed: !matches!(buffer[position], 0),
            sha_hash: None,
            hash_type: None,
            uncompressed_size: None,
            data: vec![],
        };
        position += 1;

        if res.header_version >= 2 {
            position += 20;
            res.sha_hash = Some(buffer[position - 20..position].into());
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
            if z.read_to_end(&mut data).is_err() {
                error!("Failed to decompress chunk data");
                return None;
            }
            data
        } else {
            buffer[position..].to_vec()
        };
        Some(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::ZlibEncoder;
    use std::io::Write;

    #[test]
    fn from_vec_wrong_magic() {
        let buffer = vec![0u8; 45];
        assert_eq!(Chunk::from_vec(buffer), None);
    }

    #[test]
    fn from_vec_valid_uncompressed() {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend_from_slice(&2986228386u32.to_le_bytes());
        buffer.extend_from_slice(&1u32.to_le_bytes());
        buffer.extend_from_slice(&40u32.to_le_bytes());
        buffer.extend_from_slice(&5u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&42u64.to_le_bytes());
        buffer.push(0u8);
        buffer.extend_from_slice(&[1, 2, 3, 4, 5]);

        let chunk = Chunk::from_vec(buffer).unwrap();
        assert!(chunk.guid.starts_with("00000000"));
        assert_eq!(chunk.hash, 42);
        assert_eq!(chunk.data, vec![1, 2, 3, 4, 5]);
        assert!(!chunk.compressed);
    }

    #[test]
    fn from_vec_valid_compressed() {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&[10, 20, 30]).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend_from_slice(&2986228386u32.to_le_bytes());
        buffer.extend_from_slice(&1u32.to_le_bytes());
        buffer.extend_from_slice(&40u32.to_le_bytes());
        buffer.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&42u64.to_le_bytes());
        buffer.push(1u8);
        buffer.extend_from_slice(&compressed);

        let chunk = Chunk::from_vec(buffer).unwrap();
        assert_eq!(chunk.data, vec![10, 20, 30]);
    }

    #[test]
    fn from_vec_version2_has_sha() {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend_from_slice(&2986228386u32.to_le_bytes());
        buffer.extend_from_slice(&2u32.to_le_bytes());
        buffer.extend_from_slice(&40u32.to_le_bytes());
        buffer.extend_from_slice(&5u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&42u64.to_le_bytes());
        buffer.push(0u8);
        buffer.extend_from_slice(&[0xAB; 20]);
        buffer.push(2u8);
        buffer.extend_from_slice(&[1, 2, 3, 4, 5]);

        let chunk = Chunk::from_vec(buffer).unwrap();
        assert_eq!(chunk.sha_hash, Some(vec![0xAB; 20]));
        assert_eq!(chunk.hash_type, Some(2));
    }
}
