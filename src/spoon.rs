use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use chrono::Local;
use itertools::Itertools;
use regex::Regex;
use reqwest::blocking::Client;
use thirtyfour_sync::error::WebDriverError;
use thirtyfour_sync::ElementId;
use thirtyfour_sync::WebDriverCommands;

use super::chatgpt::ChatGPT;
use super::comment::Comment;
use super::comment::CommentType;
use super::config::Config;
use super::constant;
use super::listener::{self, Listener};
use super::player::Audio;
use super::player::Player;
use super::selenium::Selenium;
use super::util;
use super::voicevox::VoiceVox;

pub struct Spoon {
    chatgpt: ChatGPT,
    voicevox: VoiceVox,
    player: Player,
    z: Selenium,

    //comments
    comment_set: HashSet<ElementId>, //records existing comments
    previous_commenter: String,      //for combo comment

    //listeners
    previous_listeners_set: HashSet<Listener>, //for `いらっしゃい`, `おかえりなさい`, `またきてね`
    previous_listeners_map: HashMap<Listener, Instant>, //for `xxx秒の滞在でした`
    cumulative_listeners: HashSet<Listener>,   //for `おかえりなさい`

    //api call
    http_client: Client,
    live_id: u64,
}

impl Spoon {
    pub fn new(config: &Config) -> Self {
        let chatgpt = ChatGPT::new(config);
        let voicevox = VoiceVox::new(config);
        let player = Player::new();

        let z = Selenium::new(
            config.selenium.webdriver_port,
            Duration::from_millis(config.selenium.implicit_timeout_ms),
            config.selenium.should_maximize_window,
        );

        Self {
            chatgpt,
            voicevox,
            player,
            z,

            comment_set: HashSet::new(),
            previous_commenter: String::new(),

            previous_listeners_set: HashSet::new(),
            previous_listeners_map: HashMap::new(),
            cumulative_listeners: HashSet::new(),

            http_client: Client::builder()
                .timeout(Some(Duration::from_millis(3000)))
                .build()
                .unwrap(),
            live_id: 0,
        }
    }

    fn log(color: &str, s: &str, timestamp: &str) {
        println!(
            "{}[{} ({})]{}{} {}{}",
            constant::COLOR_BLACK,
            Local::now().format("%H:%M:%S"),
            timestamp,
            constant::NO_COLOR,
            color,
            s,
            constant::NO_COLOR,
        );
    }

    pub fn login(
        &mut self,
        url: &str,
        twitter_id: &str,
        twitter_password: &str,
    ) -> Result<(), WebDriverError> {
        self.z.driver().get(url)?;

        self.z.click(".btn-login")?;
        self.z.click(".btn-twitter button")?;

        self.z.switch_tab(1)?;

        self.z.input("#username_or_email", twitter_id)?;
        self.z.input("#password", twitter_password)?;
        self.z.click("#allow")?;

        self.z.switch_tab(0)?;

        Ok(())
    }

    pub fn start_live(&mut self, config: &Config) -> Result<(), Box<dyn Error>> {
        let live = &config.spoon.live;
        if (!live.enabled) {
            return Ok(());
        }

        std::thread::sleep(Duration::from_millis(3000));
        self.z.driver().get(&live.start_url)?;

        //genre
        self.z.click(&format!("button[title='{}']", live.genre))?;

        //title
        self.z.input("input[name='title']", &live.title)?;

        //tags
        if (!live.tags.is_empty()) {
            self.z.click("button.btn-tag")?;
            let tags = self.z.query_all("div.input-tag-wrap input.input-tag")?;
            for (i, tag) in tags.iter().enumerate().take(live.tags.len()) {
                tag.send_keys(&live.tags[i])?;
            }
            self.z.click("button[title='確認']")?;
        }

        //pinned message
        self.z
            .input("textarea[name='welcomeMessage']", &live.pinned_comment)?;

        //background image
        //|https://stackoverflow.com/questions/11256732/how-to-handle-windows-file-upload-using-selenium-webdriver|
        if (!live.bg_image.is_empty()) {
            if (!Path::new(&live.bg_image).is_file()) {
                return Err(format!("bg image [ {} ] not found", live.bg_image).into());
            }
            self.z.input("input.input-file", &live.bg_image)?
        }

        //bgm
        if (live.bgm.enabled) {
            self.player
                .play_async(&Audio::new(&live.bgm.path, live.bgm.volume, false, true));
        }

        self.z.click("button.btn-create")?;
        std::thread::sleep(Duration::from_millis(3000));

        Ok(())
    }

    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        if let serde_json::value::Value::Number(n) = self.z.execute_javascript(
            "return JSON.parse(window.localStorage.SPOONCAST_liveBroadcastOnair).liveId;",
        )? {
            match n.as_u64() {
                Some(id) => self.live_id = id,
                None => return Err("Failed to parse the live id as number.".into()),
            }
        } else {
            return Err("Failed to retrieve the live id.".into());
        }

        Ok(())
    }

    fn get_timestamp(&self) -> Result<String, WebDriverError> {
        self.z.inner_text(".time-chip-container span")
    }

    fn post_comment(&self, s: &str) -> Result<(), WebDriverError> {
        self.z.input("textarea", s)?;
        self.z.click("button[title='送信']")?;
        Ok(())
    }

    pub fn process_comments(&mut self, config: &Config) -> Result<(), WebDriverError> {
        let timestamp = self.get_timestamp()?;

        let l = self.z.query_all("li.chat-list-item")?;

        let num_new_comment = {
            let mut c = 0;
            for e in l.iter().rev() {
                if (self.comment_set.contains(&e.element_id)) {
                    break;
                }
                self.comment_set.insert(e.element_id.clone());
                c += 1;
            }
            c
        };

        if (num_new_comment == 0) {
            return Ok(());

        //With a small enough check interval, it is unexpected `num_new_comment` has a large value.
        //However, it sometimes happened for some reason: at that time, it seemed the already processed comments in the past were mistakenly treated as new comments.
        //The cause is unknown but we suspect `element_id` may be reassigned by a bug of Spoon or Selenium.
        } else if (num_new_comment >= 15) {
            #[allow(dead_code)] //actually used in `println!()`
            #[derive(Debug)]
            struct S {
                element_id: String,
                class: String,
                inner_text: String,
            }
            println!(
                "The value of `num_new_comment` is too large: {}",
                num_new_comment
            );
            println!(
                "New comments: {:?}",
                l.iter()
                    .skip(l.len() - num_new_comment)
                    .map(|e| S {
                        element_id: e.element_id.to_string(),
                        class: e.class_name().unwrap_or_default().unwrap_or_default(),
                        inner_text: e.text().unwrap_or_default()
                    })
                    .collect_vec()
            );
            return Ok(());
        }

        for e in l.iter().skip(l.len() - num_new_comment) {
            let inner_text = match e.text() {
                Ok(s) => s,
                Err(_) => continue,
            };

            let class_name = match e.class_name() {
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
                Ok(s) => s,
            };

            match CommentType::new(class_name) {
                //match arms {{{
                CommentType::Message => {
                    let tokens = inner_text.splitn(2, '\n').collect_vec();
                    if (tokens.len() != 2) {
                        println!("Comment [ {} ] has an unexpected form.", inner_text);
                        continue;
                    }

                    let comment = Comment::new(tokens[0].to_string(), tokens[1].to_string());
                    Self::log("", &comment.to_string(), &timestamp);

                    //NOTE: This code shall sync with that in `CommentType::Combo => { ... }`.
                    //      Refactoring this as a method was difficult since `self.chatgpt.complete()` borrows self as mutable though we already borrow self in `let l = self.z.query_all("li.chat-list-item")?;`.
                    if (config.chatgpt.enabled && (comment.user() != config.chatgpt.excluded_user))
                    {
                        //`split_whitespace().join(" ")` is needed to always make a single query even when a comment is multi-line.
                        if let Some(s) = self
                            .chatgpt
                            .complete(&comment.text().split_whitespace().join(" "))
                        {
                            let s = s.trim();
                            //As each comment is truncated to at most 100 characters (in Unicode) in Spoon, we avoid information's being lost by explicitly splitting a comment.
                            for mut s in s.chars().chunks(100).into_iter() {
                                self.post_comment(&s.join(""))?;
                            }
                            if (config.voicevox.enabled) {
                                self.voicevox.say(s, false);
                            }
                        }
                    }

                    self.previous_commenter = String::from(comment.user());
                }

                CommentType::Combo => {
                    let comment = Comment::new(self.previous_commenter.clone(), inner_text);
                    Self::log("", &comment.to_string(), &timestamp);

                    //NOTE: This code shall sync with that in `CommentType::Combo => { ... }`.
                    //      Refactoring this as a method was difficult since `self.chatgpt.complete()` borrows self as mutable though we already borrow self in `let l = self.z.query_all("li.chat-list-item")?;`.
                    if (config.chatgpt.enabled && (comment.user() != config.chatgpt.excluded_user))
                    {
                        //`split_whitespace().join(" ")` is needed to always make a single query even when a comment is multi-line.
                        if let Some(s) = self
                            .chatgpt
                            .complete(&comment.text().split_whitespace().join(" "))
                        {
                            let s = s.trim();
                            //As each comment is truncated to at most 100 characters (in Unicode) in Spoon, we avoid information's being lost by explicitly splitting a comment.
                            for mut s in s.chars().chunks(100).into_iter() {
                                self.post_comment(&s.join(""))?;
                            }
                            if (config.voicevox.enabled) {
                                self.voicevox.say(s, false);
                            }
                        }
                    }
                }

                CommentType::Guide => {
                    let c = inner_text.replace("分前だよ！", "分前だよ");
                    Self::log(constant::COLOR_WHITE, &c, &timestamp);
                    if ((inner_text.contains("10分前だよ")
                        || inner_text.contains("5分前だよ")
                        || inner_text.contains("1分前だよ"))
                        && config.spoon.should_comment_guide)
                    {
                        self.post_comment(&c)?;
                        if (config.voicevox.enabled) {
                            self.voicevox.say(&c, false);
                        }
                    }

                    //点呼
                    if (inner_text.contains("1分前だよ") && config.spoon.should_call_over) {
                        let c = "点呼するよ。";
                        self.post_comment(c)?;
                        if (config.voicevox.enabled) {
                            self.voicevox.say(c, false);
                        }
                        for listener in &self.previous_listeners_set {
                            let c = format!("{}さん、来てくれてありがとう。", listener.nickname);
                            self.post_comment(&c)?;
                            if (config.voicevox.enabled) {
                                self.voicevox.say(&c, false);
                            }
                        }
                    }
                }

                CommentType::Like => {
                    let c = format!(
                        "{}さん、ハートありがとう。",
                        inner_text.replace("さんがハートを押したよ！", "")
                    );
                    Self::log(constant::COLOR_YELLOW, &c, &timestamp);
                    if (config.spoon.should_comment_heart) {
                        self.post_comment(&c)?;
                        if (config.voicevox.enabled) {
                            self.voicevox.say(&c, true);
                        }
                    }
                }

                CommentType::Present => {
                    let pat = Regex::new(r#"^([^\n]*)\n+(.*Spoon.*|ハート.*|心ばかりの粗品.*)$"#)
                        .unwrap();
                    match pat.captures(&inner_text) {
                        None => (),
                        Some(groups) => {
                            if (groups.len() != 3) {
                                println!("Present [ {} ] has an unexpected form.", inner_text);
                                continue;
                            }

                            //buster
                            if (groups.get(2).unwrap().as_str().starts_with("ハート")) {
                                Self::log(
                                    "",
                                    &format!(
                                        "{}{}:{} {}",
                                        constant::COLOR_RED,
                                        groups.get(1).unwrap().as_str(),
                                        constant::NO_COLOR,
                                        groups.get(2).unwrap().as_str(),
                                    ),
                                    &timestamp,
                                );

                                if (config.spoon.should_comment_spoon) {
                                    let s = format!(
                                        "{}さん、バスターありがとう。",
                                        groups.get(1).unwrap().as_str()
                                    );
                                    self.post_comment(&s)?;
                                    if (config.voicevox.enabled) {
                                        self.voicevox.say(&s, true);
                                    }
                                }

                            //心ばかりの粗品
                            } else if (groups
                                .get(2)
                                .unwrap()
                                .as_str()
                                .starts_with("心ばかりの粗品"))
                            {
                                Self::log(
                                    "",
                                    &format!(
                                        "{}{}:{} {}",
                                        constant::COLOR_RED,
                                        groups.get(1).unwrap().as_str(),
                                        constant::NO_COLOR,
                                        groups.get(2).unwrap().as_str(),
                                    ),
                                    &timestamp,
                                );

                                if (config.spoon.should_comment_spoon) {
                                    let s = format!(
                                        "{}さん、粗品ありがとう。",
                                        groups.get(1).unwrap().as_str(),
                                    );
                                    self.post_comment(&s)?;
                                    if (config.voicevox.enabled) {
                                        self.voicevox.say(&s, true);
                                    }
                                }

                            //spoon
                            } else {
                                Self::log(
                                    "",
                                    &format!(
                                        "{}{}:{} {}",
                                        constant::COLOR_CYAN,
                                        groups.get(1).unwrap().as_str(),
                                        constant::NO_COLOR,
                                        groups.get(2).unwrap().as_str(),
                                    ),
                                    &timestamp,
                                );

                                if (config.spoon.should_comment_spoon) {
                                    let s = format!(
                                        "{}さん、スプーンありがとう。",
                                        groups.get(1).unwrap().as_str(),
                                    );
                                    self.post_comment(&s)?;
                                    if (config.voicevox.enabled) {
                                        self.voicevox.say(&s, true);
                                    }
                                }
                            }
                        }
                    }
                }

                CommentType::Block => {
                    let c = inner_text;
                    Self::log(constant::COLOR_RED, &c, &timestamp);
                    if (config.spoon.should_comment_block) {
                        self.post_comment(&c)?;
                        if (config.voicevox.enabled) {
                            self.voicevox.say(&c, true);
                        }
                    }
                }

                CommentType::Unknown => continue,
                //}}}
            }
        }

        Ok(())
    }

    pub fn process_listeners(&mut self, config: &Config) -> Result<(), Box<dyn Error>> {
        let timestamp = self.get_timestamp()?;

        let listeners_set = listener::retrieve_listeners(&self.http_client, self.live_id)?
            .into_iter()
            .collect::<HashSet<_>>();

        let exited_listeners = &self.previous_listeners_set - &listeners_set;
        let new_listeners = &listeners_set - &self.previous_listeners_set;

        for e in exited_listeners {
            if (self.previous_listeners_map.contains_key(&e)) {
                let c = format!("{}さん、また来てね。", e.nickname);
                let c_with_time = format!(
                    "{}(滞在時間: {})",
                    c,
                    util::pretty_print_duration(
                        self.previous_listeners_map.get(&e).unwrap().elapsed()
                    )
                );
                Self::log(constant::COLOR_GREEN, &c_with_time, &timestamp);
                if (config.spoon.should_comment_listener) {
                    self.post_comment(&c_with_time)?;
                    if (config.voicevox.enabled) {
                        self.voicevox.say(&c, false);
                    }
                }
                self.previous_listeners_map.remove(&e);
            } else {
                //unexpected to happen
                let c = format!("{}さん、また来てね。", e.nickname);
                Self::log(constant::COLOR_GREEN, &c, &timestamp);
                if (config.spoon.should_comment_listener) {
                    self.post_comment(&c)?;
                    if (config.voicevox.enabled) {
                        self.voicevox.say(&c, false);
                    }
                }
            }
        }

        for e in new_listeners {
            self.previous_listeners_map
                .insert(e.clone(), Instant::now());
            if (self.cumulative_listeners.contains(&e)) {
                let c = format!("{}さん、おかえりなさい。", e.nickname);
                Self::log(constant::COLOR_GREEN, &c, &timestamp);
                if (config.spoon.should_comment_listener) {
                    self.post_comment(&c)?;
                    if (config.voicevox.enabled) {
                        self.voicevox.say(&c, false);
                    }
                }
            } else {
                self.cumulative_listeners.insert(e.clone());
                let c = format!("{}さん、いらっしゃい。", e.nickname);
                Self::log(
                    constant::COLOR_GREEN,
                    &format!("{} ({:?})", c, e), //We print `e` itself to trace the unique user id of a troll.
                    &timestamp,
                );
                if (config.spoon.should_comment_listener) {
                    self.post_comment(&c)?;
                    if (config.voicevox.enabled) {
                        self.voicevox.say(&c, false);
                    }
                }
            }
        }

        self.previous_listeners_set = listeners_set;

        Ok(())
    }

    //Sometimes you may want to manually post an arbitrary comment.
    //At that time, you can write any string to the file whose path is specified via `config.spoon.message_tunnel_file`,
    // and this function reads it and posts the content as a comment, removing the file after that.
    pub fn process_message_tunnel(&mut self, config: &Config) -> Result<(), Box<dyn Error>> {
        let p = Path::new(&config.spoon.message_tunnel_file);
        if (!p.is_file()) {
            return Ok(());
        }
        let s = std::fs::read_to_string(p)?.trim().to_string();
        std::fs::remove_file(p)?;
        if (!s.is_empty()) {
            self.post_comment(&s)?;
            if (config.voicevox.enabled) {
                self.voicevox.say(&s, false);
            }
        }
        Ok(())
    }
}
