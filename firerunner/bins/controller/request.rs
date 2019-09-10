use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub interval: u64,
    pub function: String,
    pub payload: Value,
    pub user_id: u32,
}

pub fn parse_json(json: String) -> Result<Request, serde_json::Error> {
    serde_json::from_str(json.as_str())
}
