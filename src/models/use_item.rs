use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UseItem {
    pub event: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub live_id: i64,
    pub data: Data,
    pub items: Vec<Value>,
    pub use_items: Vec<UseItem2>,
    pub update_component: UpdateComponent,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Data {
    pub user: User,
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
#[serde(rename = "use_item")]
pub struct UseItem2 {
    pub item_id: i64,
    pub combo: i64,
    pub effect: String,
    pub amount: i64,
    pub animation_type: String,
    pub images: Vec<Image>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateComponent {
    pub like: Like,
    pub listener: Value,
    pub total_listener: Value,
    pub spoon: Value,
    pub close_air_time: String,
    pub message: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Like {
    pub value: Value,
    pub combo: i64,
    pub amount: i64,
    pub sticker: Value,
}
