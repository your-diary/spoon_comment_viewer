//generated via |https://transform.tools/json-to-rust-serde|

use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveMessage {
    pub event: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub live_id: i64,
    pub data: Data,
    pub items: Vec<Value>,
    pub use_item: Vec<Value>,
    pub update_component: UpdateComponent,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Data {
    pub live: Live,
    pub user: User,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Live {
    pub author: Author,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Author {
    pub id: i64,
    pub nickname: String,
    pub gender: i64,
    pub tag: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub nickname: String,
    pub profile_url: Option<String>,
    pub gender: i64,
    pub tag: String,
    pub country: String,
    pub date_joined: String,
    pub is_dj: bool,
    pub is_fixedmng: bool,
    pub is_like: bool,
    pub is_staff: bool,
    pub is_vip: bool,
    pub present: i64,
    pub regular_score: i64,
    pub subscribed_to_dj: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateComponent {
    pub message: Message,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub value: String,
}
