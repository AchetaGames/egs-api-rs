use super::fab_search::FabTag;
use serde::{Deserialize, Serialize};

/// License type from `GET /i/taxonomy/licenses`.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabLicenseType {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub url: Option<String>,
}

/// Asset format group from `GET /i/taxonomy/asset-format-groups`.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabFormatGroup {
    pub code: Option<String>,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub extensions: Option<Vec<String>>,
}

/// Tag group from `GET /i/tags/groups`.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabTagGroup {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub tags: Option<Vec<FabTag>>,
}

/// Channel info from `GET /i/channels/{slug}`.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabChannel {
    pub uid: Option<String>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_license_type() {
        let json = r#"{"name": "Standard License", "slug": "standard", "url": "https://fab.com/licenses/standard"}"#;
        let license: FabLicenseType = serde_json::from_str(json).unwrap();
        assert_eq!(license.name, Some("Standard License".to_string()));
        assert_eq!(license.slug, Some("standard".to_string()));
        assert_eq!(
            license.url,
            Some("https://fab.com/licenses/standard".to_string())
        );
    }

    #[test]
    fn deserialize_format_group() {
        let json = r#"{"code": "ue", "name": "Unreal Engine", "icon": "ue-icon.svg", "extensions": [".uasset", ".umap"]}"#;
        let group: FabFormatGroup = serde_json::from_str(json).unwrap();
        assert_eq!(group.code, Some("ue".to_string()));
        assert_eq!(group.name, Some("Unreal Engine".to_string()));
        assert_eq!(
            group.extensions,
            Some(vec![".uasset".to_string(), ".umap".to_string()])
        );
    }

    #[test]
    fn deserialize_tag_group() {
        let json = r#"{
            "name": "Style",
            "slug": "style",
            "tags": [
                {"name": "Realistic", "slug": "realistic", "uid": "tag-1"},
                {"name": "Stylized", "slug": "stylized", "uid": "tag-2"}
            ]
        }"#;
        let group: FabTagGroup = serde_json::from_str(json).unwrap();
        assert_eq!(group.name, Some("Style".to_string()));
        let tags = group.tags.unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, Some("Realistic".to_string()));
    }

    #[test]
    fn deserialize_channel() {
        let json = r#"{
            "uid": "ch-001",
            "name": "Unreal Engine",
            "slug": "unreal-engine",
            "icon": "ue-icon.png",
            "description": "Assets for Unreal Engine",
            "coverImage": "ue-cover.jpg"
        }"#;
        let channel: FabChannel = serde_json::from_str(json).unwrap();
        assert_eq!(channel.slug, Some("unreal-engine".to_string()));
        assert_eq!(channel.cover_image, Some("ue-cover.jpg".to_string()));
    }

    #[test]
    fn deserialize_empty_fields() {
        let json = r#"{}"#;
        let license: FabLicenseType = serde_json::from_str(json).unwrap();
        assert_eq!(license.name, None);
        let group: FabFormatGroup = serde_json::from_str(json).unwrap();
        assert_eq!(group.code, None);
        let channel: FabChannel = serde_json::from_str(json).unwrap();
        assert_eq!(channel.uid, None);
    }
}
