use std::error::Error;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Listeners {
    results: Vec<Listener>,
    next: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, Hash, PartialEq)]
pub struct Listener {
    pub id: usize,        //internal user id such as `315121534`
    pub nickname: String, //username
    pub tag: String,      //user id such as `@momo_chan` but WITHOUT `@`
}

//retrieves the list of the names of current listeners
pub fn retrieve_listeners(
    http_client: &Client,
    live_id: u64,
) -> Result<Vec<Listener>, Box<dyn Error>> {
    let mut ret = Vec::with_capacity(100);

    let f = |url: &str| -> Result<Listeners, Box<dyn Error>> {
        let res = http_client.get(url).send()?.text()?;
        //Manual cast is needed here.
        //|https://stackoverflow.com/questions/57423880/handling-serde-error-and-other-error-type|
        serde_json::from_str(&res).map_err(|err| Box::new(err) as Box<dyn Error>)
    };

    let mut url = format!("https://jp-api.spooncast.net/lives/{}/listeners/", live_id);
    loop {
        let mut res = f(&url)?;
        ret.append(&mut res.results);
        if (res.next.is_empty()) {
            break;
        } else {
            url = res.next;
        }
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    // #[ignore]
    fn test01() {
        let client = Client::new();
        let live_id = 34230654;
        let l = retrieve_listeners(&client, live_id).unwrap();
        println!("l = {:?}", l);
        println!("l.len() = {}", l.len());
    }
}
