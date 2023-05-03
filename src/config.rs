use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use super::util;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub twitter: Twitter,
    pub spoon: Spoon,
    pub database_file: String,
    pub selenium: Selenium,
    pub forbidden_words: Vec<String>,
    pub voicevox: VoiceVox,
    pub chatgpt: ChatGPT,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Twitter {
    pub id: String,
    pub password: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Spoon {
    pub url: String,
    pub comment_check_interval_ms: u64,
    pub listener_check_interval_ratio: usize,
    pub should_comment_listener: bool,
    pub should_comment_heart: bool,
    pub should_comment_spoon: bool,
    pub should_comment_guide: bool,
    pub should_comment_block: bool,
    pub should_call_over: bool,
    pub message_tunnel_file: String,
    pub live: Live,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Live {
    pub enabled: bool,
    pub autostart: bool,
    pub start_url: String,
    pub genre: String,
    pub title: String,
    pub tags: Vec<String>,
    pub pinned_comment: String,
    pub bg_image: String,
    pub bgm: BGM,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct BGM {
    pub enabled: bool,
    pub audio_list: Vec<Audio>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Audio {
    pub title: String,
    pub path: String,
    pub volume: f64,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Selenium {
    pub webdriver_port: usize,
    pub implicit_timeout_ms: u64,
    pub should_maximize_window: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct VoiceVox {
    pub enabled: bool,
    pub should_skip_non_japanese: bool,
    pub should_use_google_speech_for_non_japanese: bool,
    pub url: String,
    pub api_key: String,
    pub speaker: usize,
    pub speed: f64,
    pub output_dir: String,
    pub timeout_sec: u64,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
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
        util::canonicalize_path_in_place(&mut ret.chatgpt.project_dir);
        util::canonicalize_path_in_place(&mut ret.spoon.message_tunnel_file);
        util::canonicalize_path_in_place(&mut ret.spoon.live.bg_image);
        ret.spoon.live.bgm.audio_list.iter_mut().for_each(|e| {
            util::canonicalize_path_in_place(&mut e.path);
        });
        util::canonicalize_path_in_place(&mut ret.voicevox.output_dir);
        assert!(ret.spoon.live.tags.len() <= 5);
        ret
    }
}
