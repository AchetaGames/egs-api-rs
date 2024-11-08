use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use log::{debug, error, warn};
use reqwest::Url;
use serde::{de, Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::Write;
use std::io::Read;
use std::str::FromStr;

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DownloadManifest {
    #[serde(deserialize_with = "deserialize_epic_string")]
    pub manifest_file_version: u128,
    #[serde(rename = "bIsFileData")]
    pub b_is_file_data: bool,
    #[serde(rename = "AppID", deserialize_with = "deserialize_epic_string")]
    pub app_id: u128,
    pub app_name_string: String,
    pub build_version_string: String,
    pub uninstall_action_path: Option<String>,
    pub uninstall_action_args: Option<String>,
    pub launch_exe_string: String,
    pub launch_command: String,
    pub prereq_ids: Option<Vec<String>>,
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
                Ok(str) => Ok(crate::api::utils::blob_to_num::<String>(str)),
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
                Ok(str) => {
                    let mut res = crate::api::utils::bigblob_to_num::<String>(str).to_bytes_le();
                    if res.len() < 20 {
                        res.resize(20, 0);
                    }

                    Ok(res.iter().fold(String::new(), |mut output, b| {
                        let _ = write!(output, "{b:02x}");
                        output
                    }))
                }
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
    fn chunk_dir(version: u128) -> &'static str {
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

    pub(crate) fn set_custom_field(&mut self, key: String, value: String) {
        if let Some(fields) = self.custom_fields.as_mut() {
            fields.insert(key, value);
        } else {
            self.custom_fields = Some([(key, value)].iter().cloned().collect())
        };
    }

    /// Get custom field value
    pub fn custom_field(&self, key: &str) -> Option<String> {
        match &self.custom_fields {
            Some(fields) => fields.get(key).cloned(),
            None => None,
        }
    }

    /// Get the download links from the downloaded manifest
    fn download_links(&self) -> Option<HashMap<String, Url>> {
        let url = match self.custom_field("SourceURL") {
            None => match self.custom_field("BaseUrl") {
                None => {
                    return None;
                }
                Some(urls) => {
                    let split = urls.split(',').collect::<Vec<&str>>();
                    match split.first() {
                        None => {
                            return None;
                        }
                        Some(uri) => uri.to_string(),
                    }
                }
            },
            Some(uri) => uri,
        };

        let chunk_dir = DownloadManifest::chunk_dir(self.manifest_file_version);
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
                    url,
                    chunk_dir,
                    group_num,
                    hash,
                    guid.to_uppercase()
                ))
                .unwrap(),
            );
        }
        Some(result)
    }

    /// Get list of files in the manifest
    pub fn files(&self) -> HashMap<String, FileManifestList> {
        let mut result: HashMap<String, FileManifestList> = HashMap::new();
        let links = self.download_links().unwrap_or_default();

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
        result
    }

    /// Get total size of chunks in the manifest
    pub fn total_download_size(&self) -> u128 {
        let mut total: u128 = 0;
        for size in self.chunk_filesize_list.values() {
            total += size;
        }
        total
    }

    /// Get total size of chunks in the manifest
    pub fn total_size(&self) -> u128 {
        let mut total: u128 = 0;
        for f in &self.file_manifest_list {
            total += f.size();
        }
        total
    }

    /// Parse DownloadManifest from binary data or Json
    pub fn parse(data: Vec<u8>) -> Option<DownloadManifest> {
        debug!("Attempting to parse download manifest from binary data");
        // debug!("attempted json {:?}", serde_json::from_slice::<DownloadManifest>(data.as_slice()));
        let hash = Sha1::digest(&data);
        match DownloadManifest::from_vec(data.clone()) {
            None => {
                debug!("Not binary manifest trying json");
                match serde_json::from_slice::<DownloadManifest>(data.as_slice()) {
                    Ok(mut dm) => {
                        dm.set_custom_field(
                            "DownloadedManifestHash".to_string(),
                            format!("{:x}", hash),
                        );
                        Some(dm)
                    }
                    Err(_) => None,
                }
            }
            Some(mut dm) => {
                debug!("Binary parsing successful");
                dm.set_custom_field("DownloadedManifestHash".to_string(), format!("{:x}", hash));
                Some(dm)
            }
        }
    }

    /// Creates the structure from binary data
    pub fn from_vec(mut buffer: Vec<u8>) -> Option<DownloadManifest> {
        let mut res = DownloadManifest {
            manifest_file_version: 0,
            b_is_file_data: false,
            app_id: 0,
            app_name_string: "".to_string(),
            build_version_string: "".to_string(),
            uninstall_action_path: None,
            uninstall_action_args: None,
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
        debug!("Header size: {}", header_size);
        let _size_uncompressed = crate::api::utils::read_le(&buffer, &mut position);
        let _size_compressed = crate::api::utils::read_le(&buffer, &mut position);
        position += 20;
        let sha_hash: [u8; 20] = buffer[position - 20..position].try_into().unwrap();
        let compressed = !matches!(buffer[position], 0);
        position += 1;
        let _version = crate::api::utils::read_le(&buffer, &mut position);

        buffer = if compressed {
            debug!("Uncompressing");
            let mut z = ZlibDecoder::new(&buffer[position..]);
            let mut data: Vec<u8> = Vec::new();
            z.read_to_end(&mut data).unwrap();
            if !crate::api::utils::do_vecs_match(sha_hash.as_ref(), &Sha1::digest(&data)) {
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

        res.b_is_file_data = !matches!(buffer[position], 0);
        position += 1;
        res.app_id = crate::api::utils::read_le(&buffer, &mut position) as u128;
        res.app_name_string =
            crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();
        res.build_version_string =
            crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();
        res.launch_exe_string =
            crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();
        res.launch_command =
            crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();

        let entries = crate::api::utils::read_le(&buffer, &mut position);
        let mut prereq_ids: Vec<String> = Vec::new();
        for _ in 0..entries {
            if let Some(s) = crate::api::utils::read_fstring(&buffer, &mut position) {
                prereq_ids.push(s)
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

        if data_version >= 1 {
            res.build_version_string =
                crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();
        }
        if data_version >= 2 {
            res.uninstall_action_path =
                Some(crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default());
            res.uninstall_action_args =
                Some(crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default());
        }

        debug!("Manifest end position {}", position);

        debug!(
            "Manifest metadata read length(needs to match {}): {}",
            meta_size,
            position - header_size as usize
        );

        // Chunks

        let chunk_size = crate::api::utils::read_le(&buffer, &mut position);
        debug!("Chunk size {}", chunk_size);

        let _version = buffer[position];
        debug!("version: {}", _version);
        position += 1;

        debug!("Chunk count at position: {}", position);
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
            chunk.sha_hash = buffer[position - 20..position].into();
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
                chunk.sha_hash.iter().fold(String::new(), |mut output, b| {
                    let _ = write!(output, "{b:02x}");
                    output
                }),
            );
            res.chunk_hash_list.insert(chunk.guid.clone(), chunk.hash);
            res.chunk_filesize_list.insert(
                chunk.guid.clone(),
                u128::try_from(chunk.file_size).unwrap_or_default(),
            );
            res.data_group_list.insert(
                chunk.guid,
                chunk.group_num.into(),
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

        let fm_version = buffer[position];
        debug!("File manifest version: {}", fm_version);
        position += 1;
        let count = crate::api::utils::read_le(&buffer, &mut position);

        let mut files: Vec<BinaryFileManifest> = Vec::new();
        for _ in 0..count {
            files.push(BinaryFileManifest {
                filename: crate::api::utils::read_fstring(&buffer, &mut position)
                    .unwrap_or_default(),
                symlink_target: "".to_string(),
                hash: vec![],
                hash_md5: vec![],
                hash_sha256: vec![],
                mime_type: "".to_string(),
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
            file.hash = buffer[position - 20..position].into();
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

        // File Chunks
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

        if fm_version >= 1 {
            for file in files.iter_mut() {
                let has_md5 = crate::api::utils::read_le(&buffer, &mut position);
                if has_md5 != 0 {
                    position += 16;
                    file.hash_md5 = buffer[position - 16..position].into();
                }
            }
            for file in files.iter_mut() {
                file.mime_type =
                    crate::api::utils::read_fstring(&buffer, &mut position).unwrap_or_default();
            }
        }

        if fm_version >= 2 {
            for file in files.iter_mut() {
                position += 32;
                file.hash_sha256 = buffer[position - 32..position].into();
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
                    offset: chunk.offset,
                    size: chunk.size,
                })
            }
            res.file_manifest_list.push(FileManifestList {
                filename: file.filename.clone(),
                file_hash: file.hash.iter().fold(String::new(), |mut output, b| {
                    let _ = write!(output, "{b:02x}");
                    output
                }),
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

    /// Return a vector containing the manifest data
    pub fn to_vec(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        let mut data: Vec<u8> = Vec::new();
        let mut meta: Vec<u8> = Vec::new();
        // Data version
        meta.push(if self.build_version_string.is_empty() {
            0
        } else {
            1
        });
        // Feature level
        match u32::try_from(self.manifest_file_version) {
            Ok(version) => meta.append(version.to_le_bytes().to_vec().borrow_mut()),
            Err(_) => meta.append(18u32.to_le_bytes().to_vec().borrow_mut()),
        }
        // is file data
        meta.push(0);
        // app id
        match u32::try_from(self.app_id) {
            Ok(version) => meta.append(version.to_le_bytes().to_vec().borrow_mut()),
            Err(_) => meta.append(0u32.to_le_bytes().to_vec().borrow_mut()),
        }

        meta.append(crate::api::utils::write_fstring(self.app_name_string.clone()).borrow_mut());

        meta.append(
            crate::api::utils::write_fstring(self.build_version_string.clone()).borrow_mut(),
        );

        meta.append(crate::api::utils::write_fstring(self.launch_exe_string.clone()).borrow_mut());

        meta.append(crate::api::utils::write_fstring(self.launch_command.clone()).borrow_mut());

        match &self.prereq_ids {
            None => meta.append(0u32.to_le_bytes().to_vec().borrow_mut()),
            Some(prereq_ids) => {
                meta.append(
                    (prereq_ids.len() as u32)
                        .to_le_bytes()
                        .to_vec()
                        .borrow_mut(),
                );
                for prereq_id in prereq_ids {
                    meta.append(crate::api::utils::write_fstring(prereq_id.clone()).borrow_mut());
                }
            }
        }

        meta.append(crate::api::utils::write_fstring(self.prereq_name.clone()).borrow_mut());

        meta.append(crate::api::utils::write_fstring(self.prereq_path.clone()).borrow_mut());

        meta.append(crate::api::utils::write_fstring(self.prereq_args.clone()).borrow_mut());

        if !self.build_version_string.is_empty() {
            meta.append(
                crate::api::utils::write_fstring(self.build_version_string.clone()).borrow_mut(),
            );
        }
        // Meta Size
        data.append(
            ((meta.len() + 4) as u32)
                .to_le_bytes()
                .to_vec()
                .borrow_mut(),
        );
        data.append(meta.borrow_mut());

        // Chunks

        // version
        let mut chunks: Vec<u8> = vec![0];

        // count
        chunks.append(
            (self.chunk_hash_list.len() as u32)
                .to_le_bytes()
                .to_vec()
                .borrow_mut(),
        );

        for chunk in self.chunk_hash_list.keys() {
            let subs = chunk
                .as_bytes()
                .chunks(8)
                .map(std::str::from_utf8)
                .collect::<Result<Vec<&str>, _>>()
                .unwrap();
            for g in subs {
                chunks.append(
                    u32::from_str_radix(g, 16)
                        .unwrap()
                        .to_le_bytes()
                        .to_vec()
                        .borrow_mut(),
                )
            }
        }

        // TODO: PROBABLY SORT THE CHUNKS SO WE GUARANTEE THE ORDER

        for hash in self.chunk_hash_list.values() {
            match u64::try_from(*hash) {
                Ok(h) => chunks.append(h.to_le_bytes().to_vec().borrow_mut()),
                Err(_) => chunks.append((0_u64).to_le_bytes().to_vec().borrow_mut()),
            }
        }

        for sha in self.chunk_sha_list.as_ref().unwrap().values() {
            match crate::api::utils::decode_hex(sha.as_str()) {
                Ok(mut s) => chunks.append(s.borrow_mut()),
                Err(_) => chunks.append(vec![0u8; 20].borrow_mut()),
            }
        }

        for group in self.data_group_list.values() {
            chunks.append(
                u8::try_from(*group)
                    .unwrap_or_default()
                    .to_le_bytes()
                    .to_vec()
                    .borrow_mut(),
            )
        }

        // TODO: THIS IS WRONG THIS SHOULD BE UNCOMPRESSED SIZE, CAN BE PROBABLY GOT FROM THE FILE MANIFEST
        for window in self.chunk_filesize_list.values() {
            chunks.append(
                u32::try_from(*window)
                    .unwrap_or_default()
                    .to_le_bytes()
                    .to_vec()
                    .borrow_mut(),
            )
        }
        // File Size
        for file in self.chunk_filesize_list.values() {
            chunks.append(
                i64::try_from(*file)
                    .unwrap_or_default()
                    .to_le_bytes()
                    .to_vec()
                    .borrow_mut(),
            )
        }

        // Adding chunks to data
        // add chunk size
        data.append(
            ((chunks.len() + 4) as u32)
                .to_le_bytes()
                .to_vec()
                .borrow_mut(),
        );
        data.append(chunks.borrow_mut());

        // File Manifest

        // version
        let mut files: Vec<u8> = vec![0];

        // count
        files.append(
            (self.file_manifest_list.len() as u32)
                .to_le_bytes()
                .to_vec()
                .borrow_mut(),
        );

        // Filenames
        for file in &self.file_manifest_list {
            files.append(crate::api::utils::write_fstring(file.filename.clone()).borrow_mut());
        }

        // Symlink target
        // TODO: Figure out what Epic puts in theirs
        for _ in &self.file_manifest_list {
            files.append(crate::api::utils::write_fstring("".to_string()).borrow_mut());
        }

        // hash
        for file in &self.file_manifest_list {
            match crate::api::utils::decode_hex(file.file_hash.as_str()) {
                Ok(mut s) => files.append(s.borrow_mut()),
                Err(_) => files.append(vec![0u8; 20].borrow_mut()),
            }
        }

        // flags
        // TODO: Figure out what Epic puts in theirs
        files.resize(self.file_manifest_list.len(), 0);

        // install tags
        // TODO: Figure out what Epic puts in theirs
        for _ in &self.file_manifest_list {
            files.append(0u32.to_le_bytes().to_vec().borrow_mut());
            // files.append(crate::api::utils::write_fstring("".to_string()).borrow_mut());
        }

        // File Chunks
        for file in &self.file_manifest_list {
            files.append(
                (file.file_chunk_parts.len() as u32)
                    .to_le_bytes()
                    .to_vec()
                    .borrow_mut(),
            );
            for chunk_part in &file.file_chunk_parts {
                files.append(28u32.to_le_bytes().to_vec().borrow_mut());
                let subs = chunk_part
                    .guid
                    .as_bytes()
                    .chunks(8)
                    .map(std::str::from_utf8)
                    .collect::<Result<Vec<&str>, _>>()
                    .unwrap();
                for g in subs {
                    files.append(
                        u32::from_str_radix(g, 16)
                            .unwrap()
                            .to_le_bytes()
                            .to_vec()
                            .borrow_mut(),
                    )
                }
                match u32::try_from(chunk_part.offset) {
                    Ok(offset) => files.append(offset.to_le_bytes().to_vec().borrow_mut()),
                    Err(_) => files.append(0u32.to_le_bytes().to_vec().borrow_mut()),
                }
                match u32::try_from(chunk_part.size) {
                    Ok(size) => files.append(size.to_le_bytes().to_vec().borrow_mut()),
                    Err(_) => files.append(0u32.to_le_bytes().to_vec().borrow_mut()),
                }
            }
        }

        // Adding File manifest to data
        data.append(
            ((files.len() + 4) as u32)
                .to_le_bytes()
                .to_vec()
                .borrow_mut(),
        );
        data.append(files.borrow_mut());

        // Custom Fields
        // version
        let mut custom: Vec<u8> = vec![0];

        match &self.custom_fields {
            None => {
                custom.push(0);
            }
            Some(custom_fields) => {
                // count
                custom.append(
                    (custom_fields.len() as u32)
                        .to_le_bytes()
                        .to_vec()
                        .borrow_mut(),
                );

                for key in custom_fields.keys() {
                    custom.append(crate::api::utils::write_fstring(key.to_string()).borrow_mut());
                }
                for value in custom_fields.values() {
                    custom.append(crate::api::utils::write_fstring(value.to_string()).borrow_mut());
                }
            }
        }

        // Adding Custom Feilds to data
        data.append(
            ((custom.len() + 4) as u32)
                .to_le_bytes()
                .to_vec()
                .borrow_mut(),
        );
        data.append(custom.borrow_mut());

        // FINISHING METADATA (Probably done)

        let mut hasher = Sha1::new();
        hasher.update(&data);

        // Magic
        result.append(1153351692u32.to_le_bytes().to_vec().borrow_mut());
        // Header Size
        result.append(41u32.to_le_bytes().to_vec().borrow_mut());
        // Size uncompressed
        result.append((data.len() as u32).to_le_bytes().to_vec().borrow_mut());
        // Size compressed
        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        std::io::Write::write_all(&mut z, &data).unwrap();
        let mut compressed = z.finish().unwrap();
        result.append(
            (compressed.len() as u32)
                .to_le_bytes()
                .to_vec()
                .borrow_mut(),
        );
        // Sha Hash
        result.append(hasher.finalize().to_vec().borrow_mut());
        // Stored as (Compressed)
        result.push(1);
        // Version
        result.append(18u32.to_le_bytes().to_vec().borrow_mut());
        result.append(compressed.borrow_mut());
        result
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

impl FileManifestList {
    /// Get File Size
    pub fn size(&self) -> u128 {
        self.file_chunk_parts
            .iter()
            .map(|part| part.size)
            .sum::<u128>()
    }
}

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
    hash_md5: Vec<u8>,
    hash_sha256: Vec<u8>,
    mime_type: String,
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
    #[allow(dead_code)]
    file_offset: u128,
}

#[derive(Default, Debug, Clone)]
struct BinaryChunkInfo {
    #[allow(dead_code)]
    manifest_version: u128,
    guid: String,
    hash: u128,
    sha_hash: Vec<u8>,
    group_num: u8,
    window_size: u32,
    file_size: i64,
}
