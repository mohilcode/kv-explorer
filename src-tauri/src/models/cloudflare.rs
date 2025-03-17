use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::models::kv::KVEntry;

#[derive(Serialize, Deserialize, Debug)]
pub struct CloudflareNamespace {
    pub id: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CloudflareResultInfo {
    pub count: u64,
    pub cursor: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CloudflareListResponse<T> {
    pub success: bool,
    pub errors: Vec<CloudflareError>,
    pub messages: Vec<String>,
    pub result: Vec<T>,
    pub result_info: Option<CloudflareResultInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CloudflareError {
    pub code: i32,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CloudflareKeysResponse {
    pub entries: Vec<KVEntry>,
    pub cursor: Option<String>,
    pub total: u64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CloudflareKey {
    pub name: String,
    pub expiration: Option<i64>,
    pub metadata: Option<Value>,
}