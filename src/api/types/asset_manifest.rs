use reqwest::Url;
use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetManifest {
    pub elements: Vec<Element>,
    pub platform: Option<String>,
    pub label: Option<String>,
    pub namespace: Option<String>,
    pub item_id: Option<String>,
    pub app: Option<String>,
}

impl AssetManifest {
    pub(crate) fn url_csv(&self) -> String {
        let mut res: Vec<String> = Vec::new();
        for elem in &self.elements {
            for manifest in &elem.manifests {
                res.push(manifest.uri.to_string())
            }
        }
        res.join(",")
    }
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
