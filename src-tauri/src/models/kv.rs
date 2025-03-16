use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoteConnection {
    pub account_id: String,
    pub api_token: String,
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
    #[serde(default)]
    pub count: Option<usize>,
}

pub fn default_namespace_type() -> String {
    "local".to_string()
}