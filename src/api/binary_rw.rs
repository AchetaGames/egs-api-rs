use std::convert::TryInto;

pub(crate) struct BinaryReader<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> BinaryReader<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    pub fn with_position(buffer: &'a [u8], position: usize) -> Self {
        Self { buffer, position }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_position(&mut self, pos: usize) {
        self.position = pos;
    }

    pub fn remaining(&self) -> usize {
        self.buffer.len().saturating_sub(self.position)
    }

    pub fn read_u32(&mut self) -> Option<u32> {
        if self.remaining() < 4 {
            return None;
        }
        let val = u32::from_le_bytes(
            self.buffer[self.position..self.position + 4]
                .try_into()
                .ok()?,
        );
        self.position += 4;
        Some(val)
    }

    pub fn read_i32(&mut self) -> Option<i32> {
        if self.remaining() < 4 {
            return None;
        }
        let val = i32::from_le_bytes(
            self.buffer[self.position..self.position + 4]
                .try_into()
                .ok()?,
        );
        self.position += 4;
        Some(val)
    }

    pub fn read_u64(&mut self) -> Option<u64> {
        if self.remaining() < 8 {
            return None;
        }
        let val = u64::from_le_bytes(
            self.buffer[self.position..self.position + 8]
                .try_into()
                .ok()?,
        );
        self.position += 8;
        Some(val)
    }

    pub fn read_i64(&mut self) -> Option<i64> {
        if self.remaining() < 8 {
            return None;
        }
        let val = i64::from_le_bytes(
            self.buffer[self.position..self.position + 8]
                .try_into()
                .ok()?,
        );
        self.position += 8;
        Some(val)
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        if self.remaining() < 1 {
            return None;
        }
        let val = self.buffer[self.position];
        self.position += 1;
        Some(val)
    }

    pub fn read_bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.remaining() < n {
            return None;
        }
        let slice = &self.buffer[self.position..self.position + n];
        self.position += n;
        Some(slice)
    }

    pub fn read_fstring(&mut self) -> Option<String> {
        crate::api::utils::read_fstring(self.buffer, &mut self.position)
    }

    pub fn read_guid(&mut self) -> Option<String> {
        let a = self.read_u32()?;
        let b = self.read_u32()?;
        let c = self.read_u32()?;
        let d = self.read_u32()?;
        Some(format!("{:08x}{:08x}{:08x}{:08x}", a, b, c, d))
    }

    pub fn skip(&mut self, n: usize) -> Option<()> {
        if self.remaining() < n {
            return None;
        }
        self.position += n;
        Some(())
    }
}

pub(crate) struct BinaryWriter {
    data: Vec<u8>,
}

impl BinaryWriter {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn write_u32(&mut self, val: u32) {
        self.data.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_i32(&mut self, val: i32) {
        self.data.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_u64(&mut self, val: u64) {
        self.data.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_i64(&mut self, val: i64) {
        self.data.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_u8(&mut self, val: u8) {
        self.data.push(val);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn write_fstring(&mut self, s: &str) {
        let bytes = crate::api::utils::write_fstring(s);
        self.data.extend_from_slice(&bytes);
    }

    pub fn write_guid(&mut self, guid: &str) {
        let subs: Vec<&str> = guid
            .as_bytes()
            .chunks(8)
            .map(|c| std::str::from_utf8(c).unwrap_or("00000000"))
            .collect();
        for g in subs {
            self.write_u32(u32::from_str_radix(g, 16).unwrap_or(0));
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::{BinaryReader, BinaryWriter};

    #[test]
    fn reader_read_u32() {
        let buffer = [1u8, 2, 3, 4];
        let mut reader = BinaryReader::new(&buffer);
        assert_eq!(reader.read_u32(), Some(0x04030201));
        assert_eq!(reader.position(), 4);
    }

    #[test]
    fn reader_read_u32_not_enough_bytes() {
        let buffer = [1u8, 2, 3];
        let mut reader = BinaryReader::new(&buffer);
        assert_eq!(reader.read_u32(), None);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn reader_read_i32() {
        let buffer = [237u8, 201, 255, 255];
        let mut reader = BinaryReader::new(&buffer);
        assert_eq!(reader.read_i32(), Some(-13843));
        assert_eq!(reader.position(), 4);
    }

    #[test]
    fn reader_read_u64() {
        let buffer = [0u8, 0, 5, 3, 0, 1, 2, 3];
        let mut reader = BinaryReader::new(&buffer);
        assert_eq!(reader.read_u64(), Some(216736831629492224));
        assert_eq!(reader.position(), 8);
    }

    #[test]
    fn reader_read_i64() {
        let buffer = [237u8, 201, 255, 255, 255, 255, 255, 255];
        let mut reader = BinaryReader::new(&buffer);
        assert_eq!(reader.read_i64(), Some(-13843));
        assert_eq!(reader.position(), 8);
    }

    #[test]
    fn reader_read_u8() {
        let buffer = [9u8];
        let mut reader = BinaryReader::new(&buffer);
        assert_eq!(reader.read_u8(), Some(9));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn reader_read_bytes() {
        let buffer = [1u8, 2, 3, 4, 5];
        let mut reader = BinaryReader::new(&buffer);
        assert_eq!(reader.read_bytes(3), Some(&buffer[0..3]));
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn reader_read_guid() {
        let mut writer = BinaryWriter::new();
        writer.write_u32(1);
        writer.write_u32(2);
        writer.write_u32(3);
        writer.write_u32(4);
        let data = writer.into_vec();
        let mut reader = BinaryReader::new(&data);
        assert_eq!(
            reader.read_guid(),
            Some("00000001000000020000000300000004".to_string())
        );
    }

    #[test]
    fn reader_read_fstring() {
        let mut writer = BinaryWriter::new();
        writer.write_fstring("hello");
        let data = writer.into_vec();
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read_fstring(), Some("hello".to_string()));
    }

    #[test]
    fn reader_skip() {
        let buffer = [1u8, 2, 3, 4];
        let mut reader = BinaryReader::new(&buffer);
        assert_eq!(reader.skip(2), Some(()));
        assert_eq!(reader.read_u8(), Some(3));
    }

    #[test]
    fn reader_remaining() {
        let buffer = [1u8, 2, 3];
        let mut reader = BinaryReader::new(&buffer);
        assert_eq!(reader.remaining(), 3);
        reader.read_u8();
        assert_eq!(reader.remaining(), 2);
    }

    #[test]
    fn writer_u32_roundtrip() {
        let mut writer = BinaryWriter::new();
        writer.write_u32(0x04030201);
        let data = writer.into_vec();
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read_u32(), Some(0x04030201));
    }

    #[test]
    fn writer_fstring_roundtrip() {
        let mut writer = BinaryWriter::new();
        writer.write_fstring("roundtrip");
        let data = writer.into_vec();
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read_fstring(), Some("roundtrip".to_string()));
    }

    #[test]
    fn writer_guid_roundtrip() {
        let mut writer = BinaryWriter::new();
        let guid = "00000001000000020000000300000004";
        writer.write_guid(guid);
        let data = writer.into_vec();
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read_guid(), Some(guid.to_string()));
    }
}
