//generated via |https://transform.tools/json-to-rust-serde|

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveJoin {
    pub event: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub live_id: i64,
    pub result: Result,
    pub appversion: String,
    pub useragent: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Result {
    pub code: i64,
    pub detail: String,
}
