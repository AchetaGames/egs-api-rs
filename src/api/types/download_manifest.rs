use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DownloadManifest {
    pub base_url: Option<Url>,
    pub manifest_file_version: String,
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
    pub chunk_hash_list: HashMap<String, String>,
    pub chunk_sha_list: Option<HashMap<String, String>>,
    pub data_group_list: HashMap<String, String>,
    pub chunk_filesize_list: HashMap<String, String>,
    pub custom_fields: ::serde_json::Value,
}

impl DownloadManifest {
    /// Convert numbers in the Download Manifest from little indian and %03d concatenated string
    pub fn blob_to_num(str: String) -> u64 {
        let mut num: u64 = 0;
        let mut shift: u64 = 0;
        for i in (0..str.len()).step_by(3) {
            if let Ok(n) = str[i..i + 3].parse::<u64>() {
                num += n << shift;
                shift += 8;
            }
        }
        return num;
    }

    /// Get chunk dir based on the manifest version
    fn get_chunk_dir(version: u64) -> &'static str {
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

        let chunk_dir = DownloadManifest::get_chunk_dir(DownloadManifest::blob_to_num(
            self.manifest_file_version.to_string(),
        ));
        let mut result: HashMap<String, Url> = HashMap::new();

        for (guid, hash) in self.chunk_hash_list.clone() {
            let group_num = match self.data_group_list.get(&guid) {
                None => {
                    continue;
                }
                Some(group) => DownloadManifest::blob_to_num(group.to_string()),
            };
            result.insert(
                guid.clone(),
                Url::parse(&format!(
                    "{}/{}/{:02}/{:016X}_{}.chunk",
                    url.as_str(),
                    chunk_dir,
                    group_num,
                    DownloadManifest::blob_to_num(hash),
                    guid
                ))
                .unwrap(),
            );
        }
        result
    }

    /// Get list of files in the manifest
    pub fn get_files(&self) -> HashMap<String, FileManifest> {
        let mut result: HashMap<String, FileManifest> = HashMap::new();
        let links = match self.base_url.clone() {
            None => HashMap::new(),
            Some(_) => self.get_download_links(),
        };

        for file in self.file_manifest_list.clone() {
            result.insert(
                file.filename.clone(),
                FileManifest {
                    filename: file.filename,
                    file_hash: file.file_hash,
                    file_chunk_parts: {
                        let mut temp: Vec<FileChunk> = Vec::new();
                        for part in file.file_chunk_parts {
                            temp.push(FileChunk {
                                guid: part.guid.clone(),
                                link: match links.get(&part.guid) {
                                    None => {
                                        continue;
                                    }
                                    Some(u) => u.clone(),
                                },
                                offset: DownloadManifest::blob_to_num(part.offset),
                                size: DownloadManifest::blob_to_num(part.size),
                            })
                        }
                        temp
                    },
                },
            );
        }
        return result;
    }
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileManifest {
    pub filename: String,
    pub file_hash: String,
    pub file_chunk_parts: Vec<FileChunk>,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileChunk {
    pub guid: String,
    pub link: Url,
    pub offset: u64,
    pub size: u64,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileManifestList {
    pub filename: String,
    pub file_hash: String,
    pub file_chunk_parts: Vec<FileChunkPart>,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileChunkPart {
    pub guid: String,
    pub offset: String,
    pub size: String,
}
