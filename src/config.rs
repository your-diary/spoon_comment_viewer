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
    webdriver_port: usize,
    implicit_timeout_ms: u64,
}

impl Config {
    pub fn new(config_file: &str) -> Self {
        let mut ret = Config {
            twitter_id: String::new(),
            twitter_password: String::new(),
            comment_check_interval_ms: 0,
            webdriver_port: 0,
            implicit_timeout_ms: 0,
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
                ret.webdriver_port = o.get("webdriver_port").unwrap().as_usize().unwrap();
                ret.implicit_timeout_ms = o.get("implicit_timeout_ms").unwrap().as_u64().unwrap();
            }
            _ => panic!(),
        }

        assert!(!ret.twitter_id.is_empty());
        assert!(!ret.twitter_password.is_empty());
        assert!(ret.comment_check_interval_ms != 0);
        assert!(ret.webdriver_port != 0);
        assert!(ret.implicit_timeout_ms != 0);

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

    pub fn webdriver_port(&self) -> usize {
        self.webdriver_port
    }

    pub fn implicit_timeout_ms(&self) -> u64 {
        self.implicit_timeout_ms
    }
}
