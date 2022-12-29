use std::error::Error;

use itertools::Itertools;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Listeners {
    results: Vec<Listener>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Listener {
    pub id: String,       //internal user id such as `315121534`
    pub nickname: String, //username
    pub tag: String,      //user id such as `@momo_chan` but WITHOUT `@`
}

//retrieves the list of the names of current listeners
//
//TODO: Currently, at most 34 listeners can be retrieved as we don't perform a paged call.
pub fn retrieve_listeners(
    http_client: &Client,
    live_id: u64,
) -> Result<Vec<Listener>, Box<dyn Error>> {
    let res = http_client
        .get(format!(
            "https://jp-api.spooncast.net/lives/{}/listeners/",
            live_id
        ))
        .send()?
        .text()?;
    let listeners: Listeners = serde_json::from_str(&res)?;
    Ok(listeners.results.into_iter().collect_vec())
}
