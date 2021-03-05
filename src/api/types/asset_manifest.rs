use reqwest::Url;
use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetManifest {
    pub elements: Vec<Element>,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    pub app_name: String,
    pub label_name: String,
    pub build_version: String,
    pub hash: String,
    pub manifests: Vec<Manifest>,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub uri: Url,
    pub query_params: Vec<QueryParam>,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryParam {
    pub name: String,
    pub value: String,
}
