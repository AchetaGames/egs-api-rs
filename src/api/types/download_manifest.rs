use flate2::read::ZlibDecoder;
use log::{debug, error, warn};
use num::{BigUint, Zero};
use reqwest::Url;
use serde::{de, Deserialize, Serialize};
use serde_json::json;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::io::Read;
use std::ops::Shl;
use std::str::FromStr;

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DownloadManifest {
    pub base_url: Option<Url>,
    #[serde(deserialize_with = "deserialize_epic_string")]
    pub manifest_file_version: u128,
    #[serde(rename = "bIsFileData")]
    pub b_is_file_data: bool,
    #[serde(rename = "AppID")]
    pub app_id: String,
    pub app_name_string: String,
    pub build_version_string: String,
    pub launch_exe_string: String,
    pub launch_command: String,
    pub prereq_ids: Option<Vec<::serde_json::Value>>,
    pub prereq_name: String,
    pub prereq_path: String,
    pub prereq_args: String,
    pub file_manifest_list: Vec<FileManifestList>,
    #[serde(deserialize_with = "deserialize_epic_hashmap")]
    pub chunk_hash_list: HashMap<String, u128>,
    pub chunk_sha_list: Option<HashMap<String, String>>,
    #[serde(deserialize_with = "deserialize_epic_hashmap")]
    pub data_group_list: HashMap<String, u128>,
    #[serde(deserialize_with = "deserialize_epic_hashmap")]
    pub chunk_filesize_list: HashMap<String, u128>,
    pub custom_fields: Option<HashMap<String, String>>,
}

fn deserialize_epic_string<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct JsonStringVisitor;

    impl<'de> de::Visitor<'de> for JsonStringVisitor {
        type Value = u128;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing json data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match FromStr::from_str(v) {
                Ok(str) => Ok(DownloadManifest::blob_to_num(str)),
                Err(_) => Err(de::Error::custom("Could not parse Epic Blob")),
            }
        }
    }

    deserializer.deserialize_string(JsonStringVisitor)
}

fn deserialize_epic_hash<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct JsonStringVisitor;

    impl<'de> de::Visitor<'de> for JsonStringVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing json data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match FromStr::from_str(v) {
                Ok(str) => Ok(DownloadManifest::bigblob_to_num(str)
                    .to_bytes_le()
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>()),
                Err(_) => Err(de::Error::custom("Could not parse Epic Blob")),
            }
        }
    }

    deserializer.deserialize_string(JsonStringVisitor)
}

fn deserialize_epic_hashmap<'de, D>(deserializer: D) -> Result<HashMap<String, u128>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let str_map = HashMap::<String, String>::deserialize(deserializer)?;
    let original_len = str_map.len();
    let data = {
        str_map
            .into_iter()
            .map(|(str_key, value)| match str_key.parse() {
                Ok(int_key) => Ok((int_key, DownloadManifest::blob_to_num(value))),
                Err(_) => Err({
                    de::Error::invalid_value(
                        de::Unexpected::Str(&str_key),
                        &"a non-negative integer",
                    )
                }),
            })
            .collect::<Result<HashMap<_, _>, _>>()?
    };
    // multiple strings could parse to the same int, e.g "0" and "00"
    if data.len() < original_len {
        return Err(de::Error::custom("detected duplicate integer key"));
    }
    Ok(data)
}

impl DownloadManifest {
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
                println!("num_after: {}", num);
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

    /// Get chunk dir based on the manifest version
    fn get_chunk_dir(version: u128) -> &'static str {
        if version >= 15 {
            "ChunksV4"
        } else if version >= 6 {
            "ChunksV3"
        } else if version >= 3 {
            "ChunksV2"
        } else {
            "Chunks"
        }
    }

    /// Get the download links from the downloaded manifest
    fn get_download_links(&self) -> HashMap<String, Url> {
        let url = match self.base_url.clone() {
            None => {
                return HashMap::new();
            }
            Some(uri) => uri,
        };

        let chunk_dir = DownloadManifest::get_chunk_dir(self.manifest_file_version);
        let mut result: HashMap<String, Url> = HashMap::new();

        for (guid, hash) in self.chunk_hash_list.clone() {
            let group_num = match self.data_group_list.get(&guid) {
                None => {
                    continue;
                }
                Some(group) => group,
            };
            result.insert(
                guid.clone(),
                Url::parse(&format!(
                    "{}/{}/{:02}/{:016X}_{}.chunk",
                    url.as_str(),
                    chunk_dir,
                    group_num,
                    hash,
                    guid
                ))
                .unwrap(),
            );
        }
        result
    }

    /// Get list of files in the manifest
    pub fn get_files(&self) -> HashMap<String, FileManifestList> {
        let mut result: HashMap<String, FileManifestList> = HashMap::new();
        let links = match self.base_url.clone() {
            None => HashMap::new(),
            Some(_) => self.get_download_links(),
        };

        for file in self.file_manifest_list.clone() {
            result.insert(
                file.filename.clone(),
                FileManifestList {
                    filename: file.filename,
                    file_hash: file.file_hash,
                    file_chunk_parts: {
                        let mut temp: Vec<FileChunkPart> = Vec::new();
                        for part in file.file_chunk_parts {
                            temp.push(FileChunkPart {
                                guid: part.guid.clone(),
                                link: match links.get(&part.guid) {
                                    None => {
                                        continue;
                                    }
                                    Some(u) => Some(u.clone()),
                                },
                                offset: part.offset,
                                size: part.size,
                            })
                        }
                        temp
                    },
                },
            );
        }
        return result;
    }

    /// Get total size of files in the manifest
    pub fn get_total_size(&self) -> u128 {
        let mut total: u128 = 0;
        for (_, size) in &self.chunk_filesize_list {
            total += size.clone();
        }
        total
    }

    fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
        let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
        matching == a.len() && matching == b.len()
    }

    fn split(buffer: &mut Vec<u8>, size: usize) -> Vec<u8> {
        let remains = buffer.drain(0..size).collect();
        remains
    }
    fn read_le(buffer: Vec<u8>) -> u32 {
        u32::from_le_bytes(buffer.try_into().unwrap())
    }

    fn read_le_128(buffer: Vec<u8>) -> u128 {
        u128::from_le_bytes(buffer.try_into().unwrap())
    }

    fn read_le_64_signed(buffer: Vec<u8>) -> i64 {
        i64::from_le_bytes(buffer.try_into().unwrap())
    }

    fn read_le_signed(buffer: Vec<u8>) -> i32 {
        i32::from_le_bytes(buffer.try_into().unwrap())
    }

    fn read_fstring(buffer: &mut Vec<u8>) -> Option<String> {
        let mut length = DownloadManifest::read_le_signed(DownloadManifest::split(buffer, 4));
        if length < 0 {
            length *= -2;
            Some(String::from_utf16_lossy(
                DownloadManifest::split(buffer, (length) as usize)
                    .chunks_exact(2)
                    .into_iter()
                    .map(|a| u16::from_ne_bytes([a[0], a[1]]))
                    .collect::<Vec<u16>>()
                    .as_slice(),
            ))
        } else if length > 0 {
            match std::str::from_utf8(&DownloadManifest::split(buffer, (length) as usize)) {
                Ok(s) => Some(s.to_string()),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    /// Parse DownloadManifest from binary data or Json
    pub fn parse(data: Vec<u8>) -> Option<DownloadManifest> {
        match DownloadManifest::from_vec(data.clone()) {
            None => match serde_json::from_slice::<DownloadManifest>(data.as_slice()) {
                Ok(dm) => Some(dm),
                Err(_) => None,
            },
            Some(dm) => Some(dm),
        }
    }

    /// Creates the structure from binary data
    pub fn from_vec(mut data: Vec<u8>) -> Option<DownloadManifest> {
        let mut res = DownloadManifest {
            base_url: None,
            manifest_file_version: 0,
            b_is_file_data: false,
            app_id: "".to_string(),
            app_name_string: "".to_string(),
            build_version_string: "".to_string(),
            launch_exe_string: "".to_string(),
            launch_command: "".to_string(),
            prereq_ids: None,
            prereq_name: "".to_string(),
            prereq_path: "".to_string(),
            prereq_args: "".to_string(),
            file_manifest_list: vec![],
            chunk_hash_list: HashMap::new(),
            chunk_sha_list: None,
            data_group_list: Default::default(),
            chunk_filesize_list: Default::default(),
            custom_fields: Default::default(),
        };

        let total_size = data.len();

        // Reading Header
        let magic = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));
        if magic != 1153351692 {
            error!("No header magic");
            return None;
        }
        let header_size = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));
        let _size_uncompressed = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));
        let _size_compressed = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));
        let sha_hash = DownloadManifest::split(&mut data, 20);
        let compressed = match DownloadManifest::split(&mut data, 1)[0] {
            0 => false,
            _ => true,
        };
        let _version = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));

        let mut data = if compressed {
            let mut z = ZlibDecoder::new(&*data);
            let mut data: Vec<u8> = Vec::new();
            z.read_to_end(&mut data).unwrap();
            if !DownloadManifest::do_vecs_match(&sha_hash, &Sha1::digest(&data).to_vec()) {
                error!("The extracted hash does not match");
                return None;
            }

            data
        } else {
            data
        };

        debug!(
            "Download manifest header read length(needs to match {}): {}",
            header_size,
            total_size - data.len()
        );

        // Manifest Meta

        let total_size = data.len();
        let meta_size = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));

        let data_version = DownloadManifest::split(&mut data, 1)[0];

        res.manifest_file_version =
            DownloadManifest::read_le(DownloadManifest::split(&mut data, 4)).into();

        res.b_is_file_data = match DownloadManifest::split(&mut data, 1)[0] {
            0 => false,
            _ => true,
        };
        res.app_id = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4)).to_string();
        res.app_name_string = DownloadManifest::read_fstring(&mut data).unwrap_or_default();
        res.build_version_string = DownloadManifest::read_fstring(&mut data).unwrap_or_default();
        res.launch_exe_string = DownloadManifest::read_fstring(&mut data).unwrap_or_default();
        res.launch_command = DownloadManifest::read_fstring(&mut data).unwrap_or_default();

        let entries = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));
        let mut prereq_ids: Vec<::serde_json::Value> = Vec::new();
        for _ in 0..entries {
            if let Some(s) = DownloadManifest::read_fstring(&mut data) {
                prereq_ids.push(json!(s))
            }
        }
        if prereq_ids.is_empty() {
            res.prereq_ids = None
        } else {
            res.prereq_ids = Some(prereq_ids);
        }

        res.prereq_name = DownloadManifest::read_fstring(&mut data).unwrap_or_default();
        res.prereq_path = DownloadManifest::read_fstring(&mut data).unwrap_or_default();
        res.prereq_args = DownloadManifest::read_fstring(&mut data).unwrap_or_default();

        res.build_version_string = if data_version > 0 {
            DownloadManifest::read_fstring(&mut data)
        } else {
            None
        }
        .unwrap_or_default();

        debug!(
            "Manifest metadata read length(needs to match {}): {}",
            meta_size,
            total_size - data.len()
        );

        // Chunks

        let total_size = data.len();
        let size = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));

        let _version = DownloadManifest::split(&mut data, 1)[0];

        let count = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));

        let mut chunks: Vec<BinaryChunkInfo> = Vec::new();
        for _ in 0..count {
            chunks.push(BinaryChunkInfo {
                manifest_version: res.manifest_file_version,
                guid: format!(
                    "{:08x}{:08x}{:08x}{:08x}",
                    DownloadManifest::read_le(DownloadManifest::split(&mut data, 4)),
                    DownloadManifest::read_le(DownloadManifest::split(&mut data, 4)),
                    DownloadManifest::read_le(DownloadManifest::split(&mut data, 4)),
                    DownloadManifest::read_le(DownloadManifest::split(&mut data, 4))
                ),
                hash: 0,
                sha_hash: Vec::new(),
                group_num: 0,
                window_size: 0,
                file_size: 0,
            });
        }

        for chunk in chunks.iter_mut() {
            chunk.hash = DownloadManifest::read_le_128(DownloadManifest::split(&mut data, 8));
        }
        for chunk in chunks.iter_mut() {
            chunk.sha_hash = DownloadManifest::split(&mut data, 20);
        }

        for chunk in chunks.iter_mut() {
            chunk.group_num = DownloadManifest::split(&mut data, 1)[0];
        }
        for chunk in chunks.iter_mut() {
            chunk.window_size = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));
        }
        for chunk in chunks.iter_mut() {
            chunk.file_size =
                DownloadManifest::read_le_64_signed(DownloadManifest::split(&mut data, 8));
        }

        let mut chunk_sha_list: HashMap<String, String> = HashMap::new();
        for chunk in chunks {
            chunk_sha_list.insert(
                chunk.guid.clone(),
                chunk
                    .sha_hash
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>(),
            );
            res.chunk_hash_list.insert(chunk.guid.clone(), chunk.hash);
            res.chunk_filesize_list.insert(
                chunk.guid.clone(),
                u128::try_from(chunk.file_size).unwrap_or_default(),
            );
            res.data_group_list.insert(
                chunk.guid,
                u128::try_from(chunk.group_num).unwrap_or_default(),
            );
        }
        res.chunk_sha_list = Some(chunk_sha_list);

        debug!(
            "Chunks read length(needs to match {}): {}",
            size,
            total_size - data.len()
        );

        // File Manifest
        let total_size = data.len();

        let size = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));

        let _version = DownloadManifest::split(&mut data, 1)[0];

        let count = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));

        let mut files: Vec<BinaryFileManifest> = Vec::new();
        for _ in 0..count {
            files.push(BinaryFileManifest {
                filename: DownloadManifest::read_fstring(&mut data).unwrap_or_default(),
                symlink_target: "".to_string(),
                hash: vec![],
                flags: 0,
                install_tags: vec![],
                chunk_parts: vec![],
                file_size: 0,
            });
        }

        for file in files.iter_mut() {
            file.symlink_target = DownloadManifest::read_fstring(&mut data).unwrap_or_default();
        }

        for file in files.iter_mut() {
            file.hash = DownloadManifest::split(&mut data, 20);
        }

        for file in files.iter_mut() {
            file.flags = DownloadManifest::split(&mut data, 1)[0];
        }

        for file in files.iter_mut() {
            let elem_count = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));
            for _ in 0..elem_count {
                file.install_tags
                    .push(DownloadManifest::read_fstring(&mut data).unwrap_or_default())
            }
        }

        for file in files.iter_mut() {
            let elem_count = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));
            let mut offset = 0;
            for _ in 0..elem_count {
                let total = data.len();
                let chunk_size = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));
                let chunk = BinaryChunkPart {
                    guid: format!(
                        "{:08x}{:08x}{:08x}{:08x}",
                        DownloadManifest::read_le(DownloadManifest::split(&mut data, 4)),
                        DownloadManifest::read_le(DownloadManifest::split(&mut data, 4)),
                        DownloadManifest::read_le(DownloadManifest::split(&mut data, 4)),
                        DownloadManifest::read_le(DownloadManifest::split(&mut data, 4))
                    ),
                    offset: DownloadManifest::read_le(DownloadManifest::split(&mut data, 4)),
                    size: DownloadManifest::read_le(DownloadManifest::split(&mut data, 4)),
                    file_offset: offset,
                };
                offset += chunk.size;
                let diff = total - data.len() - chunk_size as usize;
                if diff > 0 {
                    warn!("Did not read the entire chunk part!");
                    DownloadManifest::split(&mut data, diff);
                }
                file.chunk_parts.push(chunk);
            }
        }

        for file in files.iter_mut() {
            file.file_size = file.chunk_parts.iter().map(|chunk| chunk.size).sum();
        }

        for file in files.iter_mut() {
            let mut chunks: Vec<FileChunkPart> = Vec::new();
            for chunk in &file.chunk_parts {
                chunks.push(FileChunkPart {
                    guid: chunk.guid.clone(),
                    link: None,
                    offset: chunk.offset as u128,
                    size: chunk.size as u128,
                })
            }
            res.file_manifest_list.push(FileManifestList {
                filename: file.filename.clone(),
                file_hash: file
                    .hash
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>(),
                file_chunk_parts: chunks,
            })
        }

        debug!(
            "File Manifests read length(needs to match {}): {}",
            size,
            total_size - data.len()
        );

        // Custom Fields
        let total_size = data.len();

        let size = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));

        let _version = DownloadManifest::split(&mut data, 1)[0];

        let count = DownloadManifest::read_le(DownloadManifest::split(&mut data, 4));

        let mut keys: Vec<String> = Vec::new();
        let mut values: Vec<String> = Vec::new();

        for _ in 0..count {
            keys.push(DownloadManifest::read_fstring(&mut data).unwrap_or_default());
        }

        for _ in 0..count {
            values.push(DownloadManifest::read_fstring(&mut data).unwrap_or_default());
        }

        let mut custom_fields: HashMap<String, String> = HashMap::new();
        for i in 0..count {
            custom_fields.insert(keys[i as usize].clone(), values[i as usize].clone());
        }

        res.custom_fields = Some(custom_fields);

        debug!(
            "Custom fields read length(needs to match {}): {}",
            size,
            total_size - data.len()
        );

        if data.len() > 0 {
            warn!("We have not read some data {}", data.len());
        }

        Some(res)
    }
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileManifestList {
    pub filename: String,
    #[serde(deserialize_with = "deserialize_epic_hash")]
    pub file_hash: String,
    pub file_chunk_parts: Vec<FileChunkPart>,
}

// TODO: Write deserialization so offset and size are converted to u128
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileChunkPart {
    pub guid: String,
    pub link: Option<Url>,
    #[serde(deserialize_with = "deserialize_epic_string")]
    pub offset: u128,
    #[serde(deserialize_with = "deserialize_epic_string")]
    pub size: u128,
}

#[derive(Default, Debug, Clone)]
struct BinaryFileManifest {
    filename: String,
    symlink_target: String,
    hash: Vec<u8>,
    flags: u8,
    install_tags: Vec<String>,
    chunk_parts: Vec<BinaryChunkPart>,
    file_size: u32,
}

#[derive(Default, Debug, Clone)]
struct BinaryChunkPart {
    guid: String,
    offset: u32,
    size: u32,
    file_offset: u32,
}

#[derive(Default, Debug, Clone)]
struct BinaryChunkInfo {
    manifest_version: u128,
    guid: String,
    hash: u128,
    sha_hash: Vec<u8>,
    group_num: u8,
    window_size: u32,
    file_size: i64,
}
