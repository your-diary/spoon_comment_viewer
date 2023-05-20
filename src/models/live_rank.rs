//generated via |https://transform.tools/json-to-rust-serde|

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveRank {
    pub event: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub live_id: String,
    pub order: Order,
    pub appversion: String,
    pub useragent: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub effect: String,
    pub incrby: i64,
    pub now: String,
    pub prev: String,
    pub rt_effect: String,
    pub rt_incrby: i64,
    pub rt_now: String,
    pub rt_prev: String,
}
