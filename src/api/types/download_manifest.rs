use crate::api::binary_rw::{BinaryReader, BinaryWriter};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use log::{debug, error, warn};
use reqwest::Url;
use serde::{Deserialize, Serialize, de};
use sha1::{Digest, Sha1};
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

fn parse_header(buffer: &[u8]) -> Option<(Vec<u8>, usize, u32)> {
    let mut reader = BinaryReader::new(buffer);

    let magic = reader.read_u32()?;
    if magic != 1153351692 {
        debug!("No header magic, not a binary manifest");
        return None;
    }
    let mut header_size = reader.read_u32()?;
    debug!("Header size: {}", header_size);
    let _size_uncompressed = reader.read_u32()?;
    let _size_compressed = reader.read_u32()?;
    let sha_hash_bytes = reader.read_bytes(20)?;
    let sha_hash: [u8; 20] = match sha_hash_bytes.try_into() {
        Ok(h) => h,
        Err(_) => {
            error!("Buffer too short for SHA hash");
            return None;
        }
    };
    let compressed = !matches!(reader.read_u8()?, 0);
    let _version = reader.read_u32()?;

    let mut position_after_header = reader.position();
    let data = if compressed {
        debug!("Uncompressing");
        let mut z = ZlibDecoder::new(&buffer[reader.position()..]);
        let mut data: Vec<u8> = Vec::new();
        if z.read_to_end(&mut data).is_err() {
            error!("Failed to decompress manifest data");
            return None;
        }
        if !crate::api::utils::do_vecs_match(sha_hash.as_ref(), &Sha1::digest(&data)) {
            error!("The extracted hash does not match");
            return None;
        }
        position_after_header = 0;
        header_size = 0;
        data
    } else {
        buffer.to_vec()
    };

    debug!(
        "Download manifest header read length(needs to match {}): {}",
        header_size, position_after_header
    );

    Some((data, position_after_header, header_size))
}

fn parse_meta(reader: &mut BinaryReader<'_>, res: &mut DownloadManifest) -> u32 {
    let meta_size = reader.read_u32().unwrap_or(0);

    let data_version = reader.read_u8().unwrap_or(0);

    res.manifest_file_version = reader.read_u32().unwrap_or(0).into();

    res.b_is_file_data = !matches!(reader.read_u8().unwrap_or(0), 0);
    res.app_id = reader.read_u32().unwrap_or(0) as u128;
    res.app_name_string = reader.read_fstring().unwrap_or_default();
    res.build_version_string = reader.read_fstring().unwrap_or_default();
    res.launch_exe_string = reader.read_fstring().unwrap_or_default();
    res.launch_command = reader.read_fstring().unwrap_or_default();

    let entries = reader.read_u32().unwrap_or(0);
    let mut prereq_ids: Vec<String> = Vec::new();
    for _ in 0..entries {
        if let Some(s) = reader.read_fstring() {
            prereq_ids.push(s)
        }
    }
    if prereq_ids.is_empty() {
        res.prereq_ids = None
    } else {
        res.prereq_ids = Some(prereq_ids);
    }

    res.prereq_name = reader.read_fstring().unwrap_or_default();
    res.prereq_path = reader.read_fstring().unwrap_or_default();
    res.prereq_args = reader.read_fstring().unwrap_or_default();

    if data_version >= 1 {
        res.build_version_string = reader.read_fstring().unwrap_or_default();
    }
    if data_version >= 2 {
        res.uninstall_action_path = Some(reader.read_fstring().unwrap_or_default());
        res.uninstall_action_args = Some(reader.read_fstring().unwrap_or_default());
    }

    debug!("Manifest end position {}", reader.position());

    meta_size
}

fn parse_chunks(reader: &mut BinaryReader<'_>, res: &mut DownloadManifest) -> u32 {
    let chunk_size = reader.read_u32().unwrap_or(0);
    debug!("Chunk size {}", chunk_size);

    let _version = reader.read_u8().unwrap_or(0);
    debug!("version: {}", _version);

    debug!("Chunk count at position: {}", reader.position());
    let count = reader.read_u32().unwrap_or(0);
    debug!("Reading {} chunks", count);

    let mut chunks: Vec<BinaryChunkInfo> = Vec::new();
    for _i in 0..count {
        chunks.push(BinaryChunkInfo {
            manifest_version: res.manifest_file_version,
            guid: reader.read_guid().unwrap_or_default(),
            hash: 0,
            sha_hash: Vec::new(),
            group_num: 0,
            window_size: 0,
            file_size: 0,
        });
    }

    debug!("Reading Chunk Hashes");
    for chunk in chunks.iter_mut() {
        chunk.hash = reader.read_u64().unwrap_or(0) as u128;
    }
    debug!("Reading Chunk Sha Hashes");
    for chunk in chunks.iter_mut() {
        chunk.sha_hash = reader.read_bytes(20).unwrap_or(&[0u8; 20]).into();
    }

    debug!("Reading Chunk group nums");
    for chunk in chunks.iter_mut() {
        chunk.group_num = reader.read_u8().unwrap_or(0);
    }
    for chunk in chunks.iter_mut() {
        chunk.window_size = reader.read_u32().unwrap_or(0);
    }
    for chunk in chunks.iter_mut() {
        chunk.file_size = reader.read_i64().unwrap_or(0);
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
        res.data_group_list
            .insert(chunk.guid, chunk.group_num.into());
    }
    res.chunk_sha_list = Some(chunk_sha_list);

    chunk_size
}

fn parse_files(reader: &mut BinaryReader<'_>, res: &mut DownloadManifest) -> u32 {
    let filemanifest_size = reader.read_u32().unwrap_or(0);

    let fm_version = reader.read_u8().unwrap_or(0);
    debug!("File manifest version: {}", fm_version);
    let count = reader.read_u32().unwrap_or(0);

    let mut files: Vec<BinaryFileManifest> = Vec::new();
    for _ in 0..count {
        files.push(BinaryFileManifest {
            filename: reader.read_fstring().unwrap_or_default(),
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
        file.symlink_target = reader.read_fstring().unwrap_or_default();
    }

    for file in files.iter_mut() {
        file.hash = reader.read_bytes(20).unwrap_or(&[0u8; 20]).into();
    }

    for file in files.iter_mut() {
        file.flags = reader.read_u8().unwrap_or(0);
    }

    for file in files.iter_mut() {
        let elem_count = reader.read_u32().unwrap_or(0);
        for _ in 0..elem_count {
            file.install_tags
                .push(reader.read_fstring().unwrap_or_default())
        }
    }

    // File Chunks
    for i in 0..count {
        if let Some(file) = files.get_mut(i as usize) {
            let elem_count = reader.read_u32().unwrap_or(0);
            let mut offset: u128 = 0;
            for _i in 0..elem_count {
                let total = reader.position();
                let chunk_size = reader.read_u32().unwrap_or(0);
                let chunk = BinaryChunkPart {
                    guid: reader.read_guid().unwrap_or_default(),
                    offset: reader.read_u32().unwrap_or(0) as u128,
                    size: reader.read_u32().unwrap_or(0) as u128,
                    file_offset: offset,
                };
                offset += chunk.size;
                let diff = reader.position() - total - chunk_size as usize;
                if diff > 0 {
                    warn!("Did not read the entire chunk part!");
                    let _ = reader.skip(diff);
                }
                file.chunk_parts.push(chunk);
            }
        }
    }

    if fm_version >= 1 {
        for file in files.iter_mut() {
            let has_md5 = reader.read_u32().unwrap_or(0);
            if has_md5 != 0 {
                file.hash_md5 = reader.read_bytes(16).unwrap_or(&[0u8; 16]).into();
            }
        }
        for file in files.iter_mut() {
            file.mime_type = reader.read_fstring().unwrap_or_default();
        }
    }

    if fm_version >= 2 {
        for file in files.iter_mut() {
            file.hash_sha256 = reader.read_bytes(32).unwrap_or(&[0u8; 32]).into();
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

    filemanifest_size
}

fn parse_custom_fields(reader: &mut BinaryReader<'_>, res: &mut DownloadManifest) -> u32 {
    let size = reader.read_u32().unwrap_or(0);

    let _version = reader.read_u8().unwrap_or(0);
    let count = reader.read_u32().unwrap_or(0);

    let mut keys: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    for _ in 0..count {
        keys.push(reader.read_fstring().unwrap_or_default());
    }

    for _ in 0..count {
        values.push(reader.read_fstring().unwrap_or_default());
    }

    let mut custom_fields: HashMap<String, String> = HashMap::new();
    for i in 0..count {
        custom_fields.insert(keys[i as usize].clone(), values[i as usize].clone());
    }

    res.custom_fields = Some(custom_fields);

    size
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

    pub(crate) fn set_custom_field(&mut self, key: &str, value: &str) {
        if let Some(fields) = self.custom_fields.as_mut() {
            fields.insert(key.to_string(), value.to_string());
        } else {
            self.custom_fields = Some(
                [(key.to_string(), value.to_string())]
                    .iter()
                    .cloned()
                    .collect(),
            )
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

        for (guid, hash) in &self.chunk_hash_list {
            let group_num = match self.data_group_list.get(guid.as_str()) {
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
                    *group_num,
                    *hash,
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

        for file in &self.file_manifest_list {
            result.insert(
                file.filename.clone(),
                FileManifestList {
                    filename: file.filename.clone(),
                    file_hash: file.file_hash.clone(),
                    file_chunk_parts: {
                        let mut temp: Vec<FileChunkPart> = Vec::new();
                        for part in &file.file_chunk_parts {
                            let link = match links.get(&part.guid) {
                                None => {
                                    continue;
                                }
                                Some(u) => Some(u.clone()),
                            };
                            temp.push(FileChunkPart {
                                guid: part.guid.clone(),
                                link,
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
        match DownloadManifest::from_vec(&data) {
            None => {
                debug!("Not binary manifest trying json");
                match serde_json::from_slice::<DownloadManifest>(data.as_slice()) {
                    Ok(mut dm) => {
                        let hash_string = format!("{:x}", hash);
                        dm.set_custom_field("DownloadedManifestHash", &hash_string);
                        Some(dm)
                    }
                    Err(_) => None,
                }
            }
            Some(mut dm) => {
                debug!("Binary parsing successful");
                let hash_string = format!("{:x}", hash);
                dm.set_custom_field("DownloadedManifestHash", &hash_string);
                Some(dm)
            }
        }
    }

    /// Creates the structure from binary data
    pub fn from_vec(buffer: &[u8]) -> Option<DownloadManifest> {
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

        // Reading Header
        let (buffer, position_after_header, header_size) = parse_header(buffer)?;
        let mut reader = BinaryReader::with_position(&buffer, position_after_header);

        // Manifest Meta
        let meta_size = parse_meta(&mut reader, &mut res);

        debug!(
            "Manifest metadata read length(needs to match {}): {}",
            meta_size,
            reader.position() - header_size as usize
        );

        // Chunks
        let chunk_size = parse_chunks(&mut reader, &mut res);

        debug!(
            "Chunks read length(needs to match {}): {}",
            chunk_size,
            reader.position() - meta_size as usize - header_size as usize
        );

        // File Manifest
        let filemanifest_size = parse_files(&mut reader, &mut res);

        debug!(
            "File Manifests read length(needs to match {}): {}",
            filemanifest_size,
            reader.position() - meta_size as usize - header_size as usize - chunk_size as usize
        );

        // Custom Fields
        let size = parse_custom_fields(&mut reader, &mut res);

        debug!(
            "Custom fields read length(needs to match {}): {}",
            size,
            reader.position()
                - meta_size as usize
                - header_size as usize
                - chunk_size as usize
                - filemanifest_size as usize
        );

        if reader.position()
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

    fn write_meta(&self) -> Vec<u8> {
        let mut writer = BinaryWriter::new();
        // Data version
        writer.write_u8(if self.build_version_string.is_empty() {
            0
        } else {
            1
        });
        // Feature level
        match u32::try_from(self.manifest_file_version) {
            Ok(version) => writer.write_u32(version),
            Err(_) => writer.write_u32(18u32),
        }
        // is file data
        writer.write_u8(0);
        // app id
        match u32::try_from(self.app_id) {
            Ok(version) => writer.write_u32(version),
            Err(_) => writer.write_u32(0u32),
        }

        writer.write_fstring(&self.app_name_string);

        writer.write_fstring(&self.build_version_string);

        writer.write_fstring(&self.launch_exe_string);

        writer.write_fstring(&self.launch_command);

        match &self.prereq_ids {
            None => writer.write_u32(0u32),
            Some(prereq_ids) => {
                writer.write_u32(prereq_ids.len() as u32);
                for prereq_id in prereq_ids {
                    writer.write_fstring(prereq_id);
                }
            }
        }

        writer.write_fstring(&self.prereq_name);

        writer.write_fstring(&self.prereq_path);

        writer.write_fstring(&self.prereq_args);

        if !self.build_version_string.is_empty() {
            writer.write_fstring(&self.build_version_string);
        }
        // Meta Size
        let inner = writer.into_vec();
        let mut result = BinaryWriter::new();
        result.write_u32((inner.len() + 4) as u32);
        result.write_bytes(&inner);
        result.into_vec()
    }

    fn write_chunks(&self) -> Vec<u8> {
        // version
        let mut writer = BinaryWriter::new();
        writer.write_u8(0);

        // count
        writer.write_u32(self.chunk_hash_list.len() as u32);

        for chunk in self.chunk_hash_list.keys() {
            writer.write_guid(chunk);
        }

        // TODO: PROBABLY SORT THE CHUNKS SO WE GUARANTEE THE ORDER

        for hash in self.chunk_hash_list.values() {
            match u64::try_from(*hash) {
                Ok(h) => writer.write_u64(h),
                Err(_) => writer.write_u64(0_u64),
            }
        }

        for sha in self
            .chunk_sha_list
            .as_ref()
            .unwrap_or(&HashMap::new())
            .values()
        {
            match crate::api::utils::decode_hex(sha.as_str()) {
                Ok(s) => writer.write_bytes(&s),
                Err(_) => writer.write_bytes(&[0u8; 20]),
            }
        }

        for group in self.data_group_list.values() {
            writer.write_u8(u8::try_from(*group).unwrap_or_default());
        }

        // TODO: THIS IS WRONG THIS SHOULD BE UNCOMPRESSED SIZE, CAN BE PROBABLY GOT FROM THE FILE MANIFEST
        for window in self.chunk_filesize_list.values() {
            writer.write_u32(u32::try_from(*window).unwrap_or_default());
        }
        // File Size
        for file in self.chunk_filesize_list.values() {
            writer.write_i64(i64::try_from(*file).unwrap_or_default());
        }

        // Adding chunks to data
        // add chunk size
        let inner = writer.into_vec();
        let mut result = BinaryWriter::new();
        result.write_u32((inner.len() + 4) as u32);
        result.write_bytes(&inner);
        result.into_vec()
    }

    fn write_files(&self) -> Vec<u8> {
        // version
        let mut writer = BinaryWriter::new();
        writer.write_u8(0);

        // count
        writer.write_u32(self.file_manifest_list.len() as u32);

        // Filenames
        for file in &self.file_manifest_list {
            writer.write_fstring(&file.filename);
        }

        // Symlink target
        // TODO: Figure out what Epic puts in theirs
        for _ in &self.file_manifest_list {
            writer.write_fstring("");
        }

        // hash
        for file in &self.file_manifest_list {
            match crate::api::utils::decode_hex(file.file_hash.as_str()) {
                Ok(s) => writer.write_bytes(&s),
                Err(_) => writer.write_bytes(&[0u8; 20]),
            }
        }

        // flags
        // TODO: Figure out what Epic puts in theirs
        for _ in &self.file_manifest_list {
            writer.write_u8(0u8);
        }

        // install tags
        // TODO: Figure out what Epic puts in theirs
        for _ in &self.file_manifest_list {
            writer.write_u32(0u32);
        }

        // File Chunks
        for file in &self.file_manifest_list {
            writer.write_u32(file.file_chunk_parts.len() as u32);
            for chunk_part in &file.file_chunk_parts {
                writer.write_u32(28u32);
                writer.write_guid(&chunk_part.guid);
                match u32::try_from(chunk_part.offset) {
                    Ok(offset) => writer.write_u32(offset),
                    Err(_) => writer.write_u32(0u32),
                }
                match u32::try_from(chunk_part.size) {
                    Ok(size) => writer.write_u32(size),
                    Err(_) => writer.write_u32(0u32),
                }
            }
        }

        // Adding File manifest to data
        let inner = writer.into_vec();
        let mut result = BinaryWriter::new();
        result.write_u32((inner.len() + 4) as u32);
        result.write_bytes(&inner);
        result.into_vec()
    }

    fn write_custom_fields(&self) -> Vec<u8> {
        // version
        let mut writer = BinaryWriter::new();
        writer.write_u8(0);

        match &self.custom_fields {
            None => {
                writer.write_u8(0);
            }
            Some(custom_fields) => {
                // count
                writer.write_u32(custom_fields.len() as u32);

                for key in custom_fields.keys() {
                    writer.write_fstring(key);
                }
                for value in custom_fields.values() {
                    writer.write_fstring(value);
                }
            }
        }

        // Adding Custom Feilds to data
        let inner = writer.into_vec();
        let mut result = BinaryWriter::new();
        result.write_u32((inner.len() + 4) as u32);
        result.write_bytes(&inner);
        result.into_vec()
    }

    /// Return a vector containing the manifest data
    pub fn to_vec(&self) -> Vec<u8> {
        let mut data = BinaryWriter::new();
        data.write_bytes(&self.write_meta());
        data.write_bytes(&self.write_chunks());
        data.write_bytes(&self.write_files());
        data.write_bytes(&self.write_custom_fields());
        let data = data.into_vec();

        // FINISHING METADATA (Probably done)

        let mut hasher = Sha1::new();
        hasher.update(&data);

        // Magic
        let mut result = BinaryWriter::new();
        result.write_u32(1153351692u32);
        // Header Size
        result.write_u32(41u32);
        // Size uncompressed
        result.write_u32(data.len() as u32);
        // Size compressed
        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        std::io::Write::write_all(&mut z, &data).unwrap();
        let compressed = z.finish().unwrap();
        result.write_u32(compressed.len() as u32);
        // Sha Hash
        result.write_bytes(&hasher.finalize());
        // Stored as (Compressed)
        result.write_u8(1);
        // Version
        result.write_u32(18u32);
        result.write_bytes(&compressed);
        result.into_vec()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn chunk_dir_versions() {
        assert_eq!(DownloadManifest::chunk_dir(0), "Chunks");
        assert_eq!(DownloadManifest::chunk_dir(3), "ChunksV2");
        assert_eq!(DownloadManifest::chunk_dir(6), "ChunksV3");
        assert_eq!(DownloadManifest::chunk_dir(15), "ChunksV4");
        assert_eq!(DownloadManifest::chunk_dir(20), "ChunksV4");
    }

    #[test]
    fn custom_field_get_set() {
        let mut manifest = DownloadManifest::default();
        assert_eq!(manifest.custom_field("foo"), None);
        manifest.set_custom_field("foo", "bar");
        assert_eq!(manifest.custom_field("foo"), Some("bar".to_string()));

        let mut populated = DownloadManifest {
            custom_fields: Some(HashMap::from([("alpha".to_string(), "beta".to_string())])),
            ..DownloadManifest::default()
        };
        populated.set_custom_field("gamma", "delta");
        assert_eq!(populated.custom_field("alpha"), Some("beta".to_string()));
        assert_eq!(populated.custom_field("gamma"), Some("delta".to_string()));
    }

    #[test]
    fn total_download_size_empty() {
        let manifest = DownloadManifest::default();
        assert_eq!(manifest.total_download_size(), 0);
    }

    #[test]
    fn total_download_size_with_data() {
        let manifest = DownloadManifest {
            chunk_filesize_list: HashMap::from([
                ("a".to_string(), 100u128),
                ("b".to_string(), 200u128),
                ("c".to_string(), 300u128),
            ]),
            ..DownloadManifest::default()
        };
        assert_eq!(manifest.total_download_size(), 600);
    }

    #[test]
    fn total_size_from_file_parts() {
        let manifest = DownloadManifest {
            file_manifest_list: vec![FileManifestList {
                filename: "one.dat".to_string(),
                file_hash: "0000000000000000000000000000000000000000".to_string(),
                file_chunk_parts: vec![
                    FileChunkPart {
                        guid: "guid1".to_string(),
                        link: None,
                        offset: 0,
                        size: 10,
                    },
                    FileChunkPart {
                        guid: "guid2".to_string(),
                        link: None,
                        offset: 10,
                        size: 20,
                    },
                ],
            }],
            ..DownloadManifest::default()
        };
        assert_eq!(manifest.total_size(), 30);
    }

    #[test]
    fn file_manifest_list_size() {
        let file = FileManifestList {
            filename: "test.dat".to_string(),
            file_hash: "0000000000000000000000000000000000000000".to_string(),
            file_chunk_parts: vec![
                FileChunkPart {
                    guid: "a".to_string(),
                    link: None,
                    offset: 0,
                    size: 100,
                },
                FileChunkPart {
                    guid: "b".to_string(),
                    link: None,
                    offset: 100,
                    size: 200,
                },
                FileChunkPart {
                    guid: "c".to_string(),
                    link: None,
                    offset: 300,
                    size: 300,
                },
            ],
        };
        assert_eq!(file.size(), 600);
    }

    #[test]
    fn binary_roundtrip() {
        let guid = "00000001000000020000000300000004".to_string();
        let file_hash = "0000000000000000000000000000000000000000".to_string();
        let manifest = DownloadManifest {
            manifest_file_version: 18,
            app_name_string: "TestApp".to_string(),
            build_version_string: "1.0.0".to_string(),
            file_manifest_list: vec![FileManifestList {
                filename: "test.dat".to_string(),
                file_hash: file_hash.clone(),
                file_chunk_parts: vec![FileChunkPart {
                    guid: guid.clone(),
                    link: None,
                    offset: 0,
                    size: 65536,
                }],
            }],
            chunk_hash_list: HashMap::from([(guid.clone(), 12345u128)]),
            chunk_sha_list: Some(HashMap::from([(
                guid.clone(),
                "0000000000000000000000000000000000000000".to_string(),
            )])),
            data_group_list: HashMap::from([(guid.clone(), 1u128)]),
            chunk_filesize_list: HashMap::from([(guid.clone(), 65536u128)]),
            custom_fields: Some(HashMap::new()),
            ..DownloadManifest::default()
        };

        let manifest_vec = manifest.to_vec();
        let roundtrip = DownloadManifest::from_vec(&manifest_vec).unwrap();
        assert_eq!(roundtrip.app_name_string, "TestApp");
        assert_eq!(roundtrip.build_version_string, "1.0.0");
        assert_eq!(roundtrip.manifest_file_version, 18);
        assert_eq!(roundtrip.chunk_hash_list.len(), 1);
        assert_eq!(roundtrip.file_manifest_list.len(), 1);
        assert_eq!(roundtrip.file_manifest_list[0].filename, "test.dat");
    }

    #[test]
    fn parse_invalid_data() {
        assert_eq!(DownloadManifest::parse(vec![0, 1, 2, 3]), None);
    }

    #[test]
    fn parse_invalid_json() {
        assert_eq!(DownloadManifest::parse(b"not json".to_vec()), None);
    }

    #[test]
    fn from_vec_no_magic() {
        let buffer = vec![0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(DownloadManifest::from_vec(&buffer), None);
    }

    #[test]
    fn from_vec_too_short() {
        let buffer = vec![0x4C, 0xB4, 0xCB, 0x44];
        assert_eq!(DownloadManifest::from_vec(&buffer), None);
    }
}
