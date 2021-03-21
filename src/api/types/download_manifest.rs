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
                Ok(str) => Ok(crate::api::utils::blob_to_num(str)),
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
                Ok(str) => Ok(crate::api::utils::bigblob_to_num(str)
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
                Ok(int_key) => Ok((int_key, crate::api::utils::blob_to_num(value))),
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

    /// Parse DownloadManifest from binary data or Json
    pub fn parse(data: Vec<u8>) -> Option<DownloadManifest> {
        debug!("Attempting to parse download manifest from binary data");
        match DownloadManifest::from_vec(data.clone()) {
            None => {
                debug!("Not binary manifest trying json");
                match serde_json::from_slice::<DownloadManifest>(data.as_slice()) {
                    Ok(dm) => Some(dm),
                    Err(_) => None,
                }
            }
            Some(dm) => {
                debug!("Binary parsing successful");
                Some(dm)
            }
        }
    }

    /// Creates the structure from binary data
    pub fn from_vec(mut buffer: Vec<u8>) -> Option<DownloadManifest> {
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

        let mut position: usize = 0;

        // Reading Header
        let magic = crate::api::utils::read_le(&buffer, &mut position);
        if magic != 1153351692 {
            error!("No header magic");
            return None;
        }
        let mut header_size = crate::api::utils::read_le(&buffer, &mut position);
        let _size_uncompressed = crate::api::utils::read_le(&buffer, &mut position);
        let _size_compressed = crate::api::utils::read_le(&buffer, &mut position);
        position += 20;
        let sha_hash: [u8; 20] = buffer[position - 20..position].try_into().unwrap();
        let compressed = match buffer[position] {
            0 => false,
            _ => true,
        };
        position += 1;
        let _version = crate::api::utils::read_le(&buffer, &mut position);

        buffer = if compressed {
            let mut z = ZlibDecoder::new(&buffer[position..]);
            let mut data: Vec<u8> = Vec::new();
            z.read_to_end(&mut data).unwrap();
            if !crate::api::utils::do_vecs_match(&sha_hash.to_vec(), &Sha1::digest(&data).to_vec())
            {
                error!("The extracted hash does not match");
                return None;
            }
            position = 0;
            header_size = 0;
            data
        } else {
            buffer
        };

        debug!(
            "Download manifest header read length(needs to match {}): {}",
            header_size, position
        );

        // Manifest Meta

        let meta_size = crate::api::utils::read_le(&buffer, &mut position);

        let data_version = buffer[position];
        position += 1;

        res.manifest_file_version = crate::api::utils::read_le(&buffer, &mut position).into();

        res.b_is_file_data = match buffer[position] {
            0 => false,
            _ => true,
        };
        position += 1;
        res.app_id = crate::api::utils::read_le(&buffer, &mut position).to_string();
        res.app_name_string =
            crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();
        res.build_version_string =
            crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();
        res.launch_exe_string =
            crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();
        res.launch_command =
            crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();

        let entries = crate::api::utils::read_le(&buffer, &mut position);
        let mut prereq_ids: Vec<::serde_json::Value> = Vec::new();
        for _ in 0..entries {
            if let Some(s) = crate::api::utils::read_fstring(&buffer, &mut position) {
                prereq_ids.push(json!(s))
            }
        }
        if prereq_ids.is_empty() {
            res.prereq_ids = None
        } else {
            res.prereq_ids = Some(prereq_ids);
        }

        res.prereq_name =
            crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();
        res.prereq_path =
            crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();
        res.prereq_args =
            crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();

        res.build_version_string = if data_version > 0 {
            crate::api::utils::read_fstring(&buffer, &mut position)
        } else {
            None
        }
        .unwrap_or_default();

        debug!(
            "Manifest metadata read length(needs to match {}): {}",
            meta_size,
            position - header_size as usize
        );

        // Chunks

        let chunk_size = crate::api::utils::read_le(&buffer, &mut position);

        let _version = buffer[position];
        position += 1;

        let count = crate::api::utils::read_le(&buffer, &mut position);
        debug!("Reading {} chunks", count);

        let mut chunks: Vec<BinaryChunkInfo> = Vec::new();
        for _i in 0..count {
            chunks.push(BinaryChunkInfo {
                manifest_version: res.manifest_file_version,
                guid: format!(
                    "{:08x}{:08x}{:08x}{:08x}",
                    crate::api::utils::read_le(&buffer, &mut position),
                    crate::api::utils::read_le(&buffer, &mut position),
                    crate::api::utils::read_le(&buffer, &mut position),
                    crate::api::utils::read_le(&buffer, &mut position)
                ),
                hash: 0,
                sha_hash: Vec::new(),
                group_num: 0,
                window_size: 0,
                file_size: 0,
            });
        }

        debug!("Reading Chunk Hashes");
        for chunk in chunks.iter_mut() {
            chunk.hash = crate::api::utils::read_le_64(&buffer, &mut position) as u128;
        }
        debug!("Reading Chunk Sha Hashes");
        for chunk in chunks.iter_mut() {
            position += 20;
            chunk.sha_hash = buffer[position - 20..position].try_into().unwrap();
        }

        debug!("Reading Chunk group nums");
        for chunk in chunks.iter_mut() {
            chunk.group_num = buffer[position];
            position += 1;
        }
        for chunk in chunks.iter_mut() {
            chunk.window_size = crate::api::utils::read_le(&buffer, &mut position);
        }
        for chunk in chunks.iter_mut() {
            chunk.file_size = crate::api::utils::read_le_64_signed(&buffer, &mut position);
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
            chunk_size,
            position - meta_size as usize - header_size as usize
        );

        // File Manifest

        let filemanifest_size = crate::api::utils::read_le(&buffer, &mut position);

        let _version = buffer[position];
        position += 1;
        let count = crate::api::utils::read_le(&buffer, &mut position);

        let mut files: Vec<BinaryFileManifest> = Vec::new();
        for _ in 0..count {
            files.push(BinaryFileManifest {
                filename: crate::api::utils::read_fstring(&buffer, &mut position)
                    .unwrap_or_default(),
                symlink_target: "".to_string(),
                hash: vec![],
                flags: 0,
                install_tags: vec![],
                chunk_parts: vec![],
                file_size: 0,
            });
        }

        for file in files.iter_mut() {
            file.symlink_target =
                crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();
        }

        for file in files.iter_mut() {
            position += 20;
            file.hash = buffer[position - 20..position].try_into().unwrap();
        }

        for file in files.iter_mut() {
            file.flags = buffer[position];
            position += 1;
        }

        for file in files.iter_mut() {
            let elem_count = crate::api::utils::read_le(&buffer, &mut position);
            for _ in 0..elem_count {
                file.install_tags.push(
                    crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default(),
                )
            }
        }

        for i in 0..count {
            if let Some(file) = files.get_mut(i as usize) {
                let elem_count = crate::api::utils::read_le(&buffer, &mut position);
                let mut offset: u128 = 0;
                for _i in 0..elem_count {
                    let total = position;
                    let chunk_size = crate::api::utils::read_le(&buffer, &mut position);
                    let chunk = BinaryChunkPart {
                        guid: format!(
                            "{:08x}{:08x}{:08x}{:08x}",
                            crate::api::utils::read_le(&buffer, &mut position),
                            crate::api::utils::read_le(&buffer, &mut position),
                            crate::api::utils::read_le(&buffer, &mut position),
                            crate::api::utils::read_le(&buffer, &mut position)
                        ),
                        offset: crate::api::utils::read_le(&buffer, &mut position) as u128,
                        size: crate::api::utils::read_le(&buffer, &mut position) as u128,
                        file_offset: offset,
                    };
                    offset += chunk.size;
                    let diff = position - total - chunk_size as usize;
                    if diff > 0 {
                        warn!("Did not read the entire chunk part!");
                        position += diff
                    }
                    file.chunk_parts.push(chunk);
                }
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
            filemanifest_size,
            position - meta_size as usize - header_size as usize - chunk_size as usize
        );

        // Custom Fields

        let size = crate::api::utils::read_le(&buffer, &mut position);

        let _version = buffer[position];
        position += 1;
        let count = crate::api::utils::read_le(&buffer, &mut position);

        let mut keys: Vec<String> = Vec::new();
        let mut values: Vec<String> = Vec::new();

        for _ in 0..count {
            keys.push(crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default());
        }

        for _ in 0..count {
            values
                .push(crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default());
        }

        let mut custom_fields: HashMap<String, String> = HashMap::new();
        for i in 0..count {
            custom_fields.insert(keys[i as usize].clone(), values[i as usize].clone());
        }

        res.custom_fields = Some(custom_fields);

        debug!(
            "Custom fields read length(needs to match {}): {}",
            size,
            position
                - meta_size as usize
                - header_size as usize
                - chunk_size as usize
                - filemanifest_size as usize
        );

        if position
            - meta_size as usize
            - header_size as usize
            - chunk_size as usize
            - filemanifest_size as usize
            - size as usize
            != 0
        {
            warn!("We have not read some data ");
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
    file_size: u128,
}

#[derive(Default, Debug, Clone)]
struct BinaryChunkPart {
    guid: String,
    offset: u128,
    size: u128,
    file_offset: u128,
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
