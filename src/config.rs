use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use json;
use json::JsonValue;
use regex::Regex;

pub struct Config {
    twitter_id: String,
    twitter_password: String,
    comment_check_interval_ms: u64,
    listener_check_interval_ratio: usize,
    should_comment_listener: bool,
    should_comment_heart: bool,
    should_comment_spoon: bool,
    should_comment_guide: bool,
    webdriver_port: usize,
    implicit_timeout_ms: u64,
    chatgpt_enabled: bool,
    chatgpt_project_dir: String,
}

impl Config {
    pub fn new(config_file: &str) -> Self {
        let mut ret = Config {
            twitter_id: String::new(),
            twitter_password: String::new(),
            comment_check_interval_ms: 0,
            listener_check_interval_ratio: 0,
            should_comment_listener: false,
            should_comment_heart: false,
            should_comment_spoon: false,
            should_comment_guide: false,
            webdriver_port: 0,
            implicit_timeout_ms: 0,
            chatgpt_enabled: false,
            chatgpt_project_dir: String::new(),
        };

        let json_string: String = {
            let file: File = File::open(config_file).unwrap();

            let comment_regex = Regex::new(r#"^\s*#.*"#).unwrap();

            BufReader::new(file)
                .lines()
                .filter(|l| !comment_regex.is_match(l.as_ref().unwrap()))
                .map(|l| l.unwrap())
                .collect::<Vec<String>>()
                .join("\n")
        };

        match json::parse(&json_string).unwrap() {
            JsonValue::Object(o) => {
                ret.twitter_id = o.get("twitter_id").unwrap().as_str().unwrap().to_string();
                ret.twitter_password = o
                    .get("twitter_password")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();
                ret.comment_check_interval_ms = o
                    .get("comment_check_interval_ms")
                    .unwrap()
                    .as_u64()
                    .unwrap();
                ret.listener_check_interval_ratio = o
                    .get("listener_check_interval_ratio")
                    .unwrap()
                    .as_usize()
                    .unwrap();
                ret.should_comment_listener =
                    o.get("should_comment_listener").unwrap().as_bool().unwrap();
                ret.should_comment_heart =
                    o.get("should_comment_heart").unwrap().as_bool().unwrap();
                ret.should_comment_spoon =
                    o.get("should_comment_spoon").unwrap().as_bool().unwrap();
                ret.should_comment_guide =
                    o.get("should_comment_guide").unwrap().as_bool().unwrap();
                ret.webdriver_port = o.get("webdriver_port").unwrap().as_usize().unwrap();
                ret.implicit_timeout_ms = o.get("implicit_timeout_ms").unwrap().as_u64().unwrap();
                ret.chatgpt_enabled = o.get("chatgpt_enabled").unwrap().as_bool().unwrap();
                ret.chatgpt_project_dir = o
                    .get("chatgpt_project_dir")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .replace('~', &std::env::var("HOME").unwrap())
                    .to_string();
            }
            _ => panic!(),
        }

        assert!(!ret.twitter_id.is_empty());
        assert!(!ret.twitter_password.is_empty());
        assert!(ret.comment_check_interval_ms != 0);
        assert!(ret.listener_check_interval_ratio > 0);
        assert!(ret.webdriver_port != 0);
        assert!(ret.implicit_timeout_ms != 0);
        assert!(!ret.chatgpt_project_dir.is_empty());

        ret
    }

    pub fn twitter_id(&self) -> &str {
        &self.twitter_id
    }

    pub fn twitter_password(&self) -> &str {
        &self.twitter_password
    }

    pub fn comment_check_interval_ms(&self) -> u64 {
        self.comment_check_interval_ms
    }

    pub fn listener_check_interval_ratio(&self) -> usize {
        self.listener_check_interval_ratio
    }

    pub fn should_comment_listener(&self) -> bool {
        self.should_comment_listener
    }

    pub fn should_comment_heart(&self) -> bool {
        self.should_comment_heart
    }

    pub fn should_comment_spoon(&self) -> bool {
        self.should_comment_spoon
    }

    pub fn should_comment_guide(&self) -> bool {
        self.should_comment_guide
    }

    pub fn webdriver_port(&self) -> usize {
        self.webdriver_port
    }

    pub fn implicit_timeout_ms(&self) -> u64 {
        self.implicit_timeout_ms
    }

    pub fn chatgpt_enabled(&self) -> bool {
        self.chatgpt_enabled
    }

    pub fn chatgpt_project_dir(&self) -> &str {
        &self.chatgpt_project_dir
    }
}
