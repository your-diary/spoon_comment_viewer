use std::thread;
use std::{error::Error, path::Path, rc::Rc, time::Duration};

use itertools::Itertools;
use log::info;
use reqwest::blocking::Client;
use thirtyfour_sync::error::WebDriverError;

use super::listener::{Listener, Listeners};
use super::selenium::Selenium;

pub struct Spoon {
    z: Rc<Selenium>,
    http_client: Client,
    live_id: u64,
}

impl Spoon {
    pub fn new(selenium: Rc<Selenium>, api_timeout: Duration) -> Self {
        Self {
            z: selenium,
            http_client: Client::builder()
                .timeout(Some(api_timeout))
                .build()
                .unwrap(),
            live_id: 0,
        }
    }

    pub fn login(
        &self,
        url: &str,
        twitter_id: &str,
        twitter_password: &str,
    ) -> Result<(), WebDriverError> {
        self.z.get(url)?;

        self.z.click("button[title='ログイン / 会員登録']")?;
        self.z.click(".btn-twitter button")?;

        self.z.switch_tab(1)?;

        self.z.input("#username_or_email", twitter_id)?;
        self.z.input("#password", twitter_password)?;
        self.z.click("#allow")?;

        self.z.switch_tab(0)?;

        Ok(())
    }

    pub fn prepare_live(
        &self,
        start_url: &str,
        genre: &str,
        title: &str,
        tags: &[String],
        pinned_comment: &str,
        bg_image: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        self.z.get(start_url)?;

        //genre
        self.z.click(&format!("button[title='{}']", genre))?;

        //title
        self.z.input("input[name='title']", title)?;

        //tags
        if (!tags.is_empty()) {
            if (tags.len() > 5) {
                return Err("at most 5 tags can be specified".into());
            }
            self.z.click("button.btn-tag")?;
            let tag_elements = self.z.query_all("div.input-tag-wrap input.input-tag")?;
            for i in 0..tags.len() {
                tag_elements[i].send_keys(&tags[i])?;
            }
            self.z.click("button[title='確認']")?;
        }

        //pinned message
        self.z
            .input("textarea[name='welcomeMessage']", pinned_comment)?;

        //background image
        //|https://stackoverflow.com/questions/11256732/how-to-handle-windows-file-upload-using-selenium-webdriver|
        if let Some(bg_image) = bg_image {
            if (!Path::new(bg_image).is_file()) {
                return Err(format!("bg image [ {} ] not found", bg_image).into());
            }
            self.z.input("input.input-file", bg_image)?
        }

        Ok(())
    }

    pub fn start_live(&self) -> Result<(), Box<dyn Error>> {
        self.z.click("button.btn-create").map_err(|e| e.into())
    }

    pub fn update_live_id(&mut self) -> Result<u64, Box<dyn Error>> {
        let f = || {
            self.z.execute_javascript(
                "return JSON.parse(window.localStorage.SPOONCAST_liveBroadcastOnair).liveId;",
            )
        };
        let max_retry = 3;
        for i in 0..max_retry {
            let live_id_json = f();
            if let Err(e) = live_id_json {
                if (i == max_retry - 1) {
                    return Err(e.into());
                } else {
                    info!("Retrying to retrieve the liveId...");
                    thread::sleep(Duration::from_millis(5000));
                    continue;
                }
            }
            if let serde_json::value::Value::Number(n) = live_id_json.unwrap() {
                match n.as_u64() {
                    Some(id) => {
                        self.live_id = id;
                        info!("live_id: {}", self.live_id);
                        return Ok(self.live_id);
                    }
                    None => return Err("Failed to parse the live id as number.".into()),
                }
            } else {
                return Err("Failed to retrieve the live id.".into());
            }
        }
        unreachable!();
    }

    pub fn post_comment(&self, s: &str) -> Result<(), WebDriverError> {
        //As each comment is truncated to at most 100 characters (in Unicode) in Spoon, we avoid information's being lost by explicitly splitting a comment.
        for mut s in s.chars().chunks(100).into_iter() {
            let s = s.join("");
            self.z.input("textarea", &s)?;
            self.z.click("button[title='送信']")?;
        }
        Ok(())
    }

    //retrieves the list of the names of current listeners
    pub fn retrieve_listeners(&self) -> Result<Vec<Listener>, Box<dyn Error>> {
        let mut ret = Vec::with_capacity(100);

        let f = |url: &str| -> Result<Listeners, Box<dyn Error>> {
            let res = self.http_client.get(url).send()?.text()?;
            //Manual cast is needed here.
            //|https://stackoverflow.com/questions/57423880/handling-serde-error-and-other-error-type|
            serde_json::from_str(&res).map_err(|err| err.into())
        };

        let mut url = format!(
            "https://jp-api.spooncast.net/lives/{}/listeners/",
            self.live_id
        );
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
}
