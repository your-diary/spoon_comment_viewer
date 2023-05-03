use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Listeners {
    pub results: Vec<Listener>,
    pub next: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, Hash, PartialEq)]
pub struct Listener {
    pub id: usize,        //internal user id such as `315121534`
    pub nickname: String, //username
    pub tag: String,      //user id such as `@momo_chan` but WITHOUT `@`
}
