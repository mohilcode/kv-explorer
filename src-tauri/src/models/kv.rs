use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoteConnection {
    pub account_id: String,
    pub api_token: String,
}

#[derive(Debug, Clone)]
pub struct LocalFolder {
    pub id: i64,
    pub path: PathBuf,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalFolderInfo {
    pub id: i64,
    pub path: String,
    pub name: String,
}

impl From<&LocalFolder> for LocalFolderInfo {
    fn from(folder: &LocalFolder) -> Self {
        LocalFolderInfo {
            id: folder.id,
            path: folder.path.to_string_lossy().to_string(),
            name: folder.name.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KVEntry {
    pub key: String,
    pub blob_id: String,
    pub expiration: Option<i64>,
    pub metadata: Option<String>,
    pub value: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KVNamespace {
    pub id: String,
    pub entries: Vec<KVEntry>,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_namespace_type")]
    pub r#type: String,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default, rename = "folderId")]
    pub folder_id: Option<i64>,
    #[serde(default)]
    pub count: Option<usize>,
}

pub fn default_namespace_type() -> String {
    "local".to_string()
}