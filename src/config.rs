use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use super::util;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub twitter: Twitter,
    pub spoon: Spoon,
    pub selenium: Selenium,
    pub coefont: CoeFont,
    pub chatgpt: ChatGPT,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Twitter {
    pub id: String,
    pub password: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Spoon {
    pub url: String,
    pub comment_check_interval_ms: u64,
    pub listener_check_interval_ratio: usize,
    pub should_comment_listener: bool,
    pub should_comment_heart: bool,
    pub should_comment_spoon: bool,
    pub should_comment_guide: bool,
    pub message_tunnel_file: String,
    pub live: Live,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Live {
    pub enabled: bool,
    pub start_url: String,
    pub genre: String,
    pub title: String,
    pub tags: Vec<String>,
    pub pinned_comment: String,
    pub bg_image: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Selenium {
    pub webdriver_port: usize,
    pub implicit_timeout_ms: u64,
    pub should_maximize_window: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CoeFont {
    pub enabled: bool,
    pub binary_path: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ChatGPT {
    pub enabled: bool,
    pub project_dir: String,
    pub excluded_user: String,
}

impl Config {
    pub fn new(config_file: &str) -> Self {
        let json_string: String = {
            let file = File::open(config_file).unwrap();
            let comment_regex = Regex::new(r#"^\s*#.*"#).unwrap();
            BufReader::new(file)
                .lines()
                .filter(|l| !comment_regex.is_match(l.as_ref().unwrap()))
                .map(|l| l.unwrap())
                .collect::<Vec<String>>()
                .join("\n")
        };

        let mut ret: Self = serde_json::from_str(&json_string).unwrap();
        ret.chatgpt.project_dir = util::tilde_expansion(&ret.chatgpt.project_dir);
        ret.spoon.message_tunnel_file = util::tilde_expansion(&ret.spoon.message_tunnel_file);
        ret.spoon.live.bg_image = util::tilde_expansion(&ret.spoon.live.bg_image);
        ret.coefont.binary_path = util::tilde_expansion(&ret.coefont.binary_path);
        assert!(ret.spoon.live.tags.len() <= 5);
        ret
    }
}
