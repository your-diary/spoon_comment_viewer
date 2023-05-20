use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LivePresent {
    pub event: String,
    pub live_id: i64,
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Data {
    pub live: Live,
    pub author: Author2,
    pub item_template_id: i64,
    pub amount: i64,
    pub combo: i64,
    pub sticker: String,
    pub sticker_type: i64,
    pub donation_msg: String,
    pub donation_audio: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Live {
    pub author: Author,
    pub broadcast_extension_count: i64,
    pub categories: Vec<String>,
    pub close_air_time: String,
    pub closed: Value,
    pub created: String,
    pub donation: i64,
    pub engine_name: String,
    pub hashtags: Vec<Hashtag>,
    pub id: i64,
    pub img_url: String,
    pub is_adult: bool,
    pub is_beginner: bool,
    pub is_call: bool,
    pub is_editors: bool,
    pub is_freeze: bool,
    pub is_join_now: bool,
    pub is_like: bool,
    pub is_live_call: bool,
    pub is_mute: bool,
    pub is_save: bool,
    pub is_vip: bool,
    pub manager_ids: Vec<Value>,
    pub msg_interval: i64,
    pub room_token: String,
    pub protocol: String,
    pub stream_name: String,
    pub sv: String,
    pub tags: Vec<String>,
    pub tier: Value,
    pub title: String,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub url: String,
    pub url_hls: String,
    pub welcome_message: String,
    pub top_fans: Vec<TopFan>,
    pub spoon_aim: Vec<Value>,
    pub system: Value,
    pub status: i64,
    pub close_status: i64,
    pub like_count: i64,
    pub member_count: i64,
    pub total_member_count: i64,
    pub total_spoon_count: i64,
    pub login_count: i64,
    pub total_login_count: i64,
    pub is_virtual: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Author {
    pub id: i64,
    pub nickname: String,
    pub profile_url: Option<String>,
    pub gender: i64,
    pub tag: String,
    pub country: String,
    pub date_joined: String,
    pub follower_count: i64,
    pub following_count: i64,
    pub follow_status: i64,
    pub is_staff: bool,
    pub is_vip: bool,
    pub top_impressions: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hashtag {
    pub id: i64,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopFan {
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
pub struct Author2 {
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
