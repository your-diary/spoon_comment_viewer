use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use chrono::Local;
use itertools::Itertools;
use regex::Regex;
use thirtyfour_sync::error::WebDriverError;
use thirtyfour_sync::ElementId;
use thirtyfour_sync::WebDriverCommands;

use super::chatgpt::ChatGPT;
use super::comment::Comment;
use super::comment::CommentType;
use super::config::Config;
use super::constant;
use super::selenium::Selenium;
use super::util;

pub struct Spoon {
    chatgpt: ChatGPT,
    z: Selenium,

    //comments
    comment_set: HashSet<ElementId>, //records existing comments
    previous_commenter: String,      //for combo comment

    //listeners
    previous_listeners_set: HashSet<String>, //for `いらっしゃい`, `おかえりなさい`, `またきてね`
    previous_listeners_map: HashMap<String, Instant>, //for `xxx秒の滞在でした`
    cumulative_listeners: HashSet<String>,   //for `おかえりなさい`
}

impl Spoon {
    pub fn new(config: &Config) -> Self {
        let chatgpt = ChatGPT::new(config);

        let z = Selenium::new(
            config.selenium.webdriver_port,
            Duration::from_millis(config.selenium.implicit_timeout_ms),
        );

        Self {
            chatgpt,
            z,

            comment_set: HashSet::new(),
            previous_commenter: String::new(),

            previous_listeners_set: HashSet::new(),
            previous_listeners_map: HashMap::new(),
            cumulative_listeners: HashSet::new(),
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
        std::thread::sleep(Duration::from_millis(500));

        Ok(())
    }

    pub fn start_live(&mut self, config: &Config) -> Result<(), WebDriverError> {
        let live = &config.spoon.live;
        if (!live.enabled) {
            return Ok(());
        }

        self.z.driver().get(&live.start_url)?;

        self.z.click(&format!("button[title='{}']", live.genre))?;
        self.z.input("input[name='title']", &live.title)?;
        if (!live.tags.is_empty()) {
            self.z.click("button.btn-tag")?;
            let tags = self.z.query_all("div.input-tag-wrap input.input-tag")?;
            for (i, tag) in tags.iter().enumerate().take(live.tags.len()) {
                tag.send_keys(&live.tags[i])?;
            }
            self.z.click("button[title='確認']")?;
        }
        self.z
            .input("textarea[name='welcomeMessage']", &live.pinned_comment)?;

        self.z.click("button.btn_create")?;

        Ok(())
    }

    pub fn init(&self) {
        //tries to open the listeners tab in the sidebar
        //We intentionally ignore the result as this operation fails when the tab is already open.
        let _ = self.z.click("button[title='リスナー']");
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
                    //      Refactoring this as a method was difficult since `self.chatgpt.complete_and_say()` borrows self as mutable though we already borrow self in `let l = self.z.query_all("li.chat-list-item")?;`.
                    if (config.chatgpt.enabled && (comment.user() != config.chatgpt.excluded_user))
                    {
                        //`split_whitespace().join(" ")` is needed to always make a single query even when a comment is multi-line.
                        if let Some(s) = self
                            .chatgpt
                            .complete_and_say(&comment.text().split_whitespace().join(" "))
                        {
                            //As each comment is truncated to at most 100 characters (in Unicode) in Spoon, we avoid information's being lost by explicitly splitting a comment.
                            for mut s in s.trim().chars().chunks(100).into_iter() {
                                self.post_comment(&s.join(""))?;
                            }
                        }
                    }

                    self.previous_commenter = String::from(comment.user());
                }

                CommentType::Combo => {
                    let comment = Comment::new(self.previous_commenter.clone(), inner_text);
                    Self::log("", &comment.to_string(), &timestamp);

                    //NOTE: This code shall sync with that in `CommentType::Combo => { ... }`.
                    //      Refactoring this as a method was difficult since `self.chatgpt.complete_and_say()` borrows self as mutable though we already borrow self in `let l = self.z.query_all("li.chat-list-item")?;`.
                    if (config.chatgpt.enabled && (comment.user() != config.chatgpt.excluded_user))
                    {
                        //`split_whitespace().join(" ")` is needed to always make a single query even when a comment is multi-line.
                        if let Some(s) = self
                            .chatgpt
                            .complete_and_say(&comment.text().split_whitespace().join(" "))
                        {
                            //As each comment is truncated to at most 100 characters (in Unicode) in Spoon, we avoid information's being lost by explicitly splitting a comment.
                            for mut s in s.trim().chars().chunks(100).into_iter() {
                                self.post_comment(&s.join(""))?;
                            }
                        }
                    }
                }

                CommentType::Guide => {
                    let c = inner_text.replace("分前だよ！", "分前だよ");
                    Self::log(constant::COLOR_WHITE, &c, &timestamp);
                    if (inner_text.contains("分前だよ") && config.spoon.should_comment_guide) {
                        self.post_comment(&c)?;
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
                    }
                }

                CommentType::Present => {
                    let pat = Regex::new(r#"^([^\n]*)\n+(.*Spoon.*|ハート.*)$"#).unwrap();
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
                                    self.post_comment(&format!(
                                        "{}さん、バスターありがとう。",
                                        groups.get(1).unwrap().as_str(),
                                    ))?;
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
                                    self.post_comment(&format!(
                                        "{}さん、スプーンありがとう。",
                                        groups.get(1).unwrap().as_str(),
                                    ))?;
                                }
                            }
                        }
                    }
                }
                CommentType::Unknown => continue,
                //}}}
            }
        }

        Ok(())
    }

    pub fn process_listeners(&mut self, config: &Config) -> Result<(), WebDriverError> {
        let timestamp = self.get_timestamp()?;

        self.z
            .click("div.user-list-wrap button[title='再読み込み']")?;

        //retrieves the list of the names of current listeners
        //
        //We can instead GET `https://jp-api.spooncast.net/lives/<live_id>/listeners/` to retrieve
        // the list of listeners where `<live_id>` can be extracted from `SPOONCAST_JP_liveCurrentInfo`
        // in local storage.
        //It is of the form `{"30538814":{"uId":"l63m46d6","created":"2022-07-27T11:30:12.193915Z"}}`.
        //
        //TODO: Currently, at most 34 listeners can be retrieved as we don't perform a paged call.
        let listeners_set: HashSet<String> = {
            //creates `listeners_set` {{{
            let mut listeners_list = Vec::new();

            //temporarily sets a small implicit wait value
            //Without this, we end up waiting long for `query_all()` to return when there is no listener.
            self.z
                .driver()
                .set_implicit_wait_timeout(Duration::from_millis(100))?;

            //`未ログインユーザー<n>人` is not included as it's not a button.
            let l = match self.z.query_all("button p.name.text-box") {
                Err(e) => {
                    self.z
                        .driver()
                        .set_implicit_wait_timeout(Duration::from_millis(
                            config.selenium.implicit_timeout_ms,
                        ))?;
                    return Err(e);
                }
                Ok(o) => {
                    self.z
                        .driver()
                        .set_implicit_wait_timeout(Duration::from_millis(
                            config.selenium.implicit_timeout_ms,
                        ))?;
                    o
                }
            };

            for e in l {
                match e.text() {
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                    Ok(s) => listeners_list.push(s),
                }
            }

            HashSet::from_iter(listeners_list.into_iter())
            //}}}
        };

        let exited_listeners = &self.previous_listeners_set - &listeners_set;
        let new_listeners = &listeners_set - &self.previous_listeners_set;

        for e in exited_listeners {
            if (self.previous_listeners_map.contains_key(&e)) {
                let c = format!(
                    "{}さん、また来てね。(滞在時間: {})",
                    e,
                    util::pretty_print_duration(
                        self.previous_listeners_map.get(&e).unwrap().elapsed()
                    ),
                );
                Self::log(constant::COLOR_GREEN, &c, &timestamp);
                if (config.spoon.should_comment_listener) {
                    self.post_comment(&c)?;
                }
                self.previous_listeners_map.remove(&e);
            } else {
                //unexpected to happen
                let c = format!("{}さん、また来てね。", e);
                Self::log(constant::COLOR_GREEN, &c, &timestamp);
                if (config.spoon.should_comment_listener) {
                    self.post_comment(&c)?;
                }
            }
        }

        for e in new_listeners {
            self.previous_listeners_map
                .insert(e.clone(), Instant::now());
            if (self.cumulative_listeners.contains(&e)) {
                let c = format!("{}さん、おかえりなさい。", e);
                Self::log(constant::COLOR_GREEN, &c, &timestamp);
                if (config.spoon.should_comment_listener) {
                    self.post_comment(&c)?;
                }
            } else {
                self.cumulative_listeners.insert(e.clone());
                let c = format!("{}さん、いらっしゃい。", e);
                Self::log(constant::COLOR_GREEN, &c, &timestamp);
                if (config.spoon.should_comment_listener) {
                    self.post_comment(&c)?;
                }
            }
        }

        self.previous_listeners_set = listeners_set;

        Ok(())
    }

    //Sometimes you may want to manually post an arbitrary comment.
    //At that time, you can write any string to the file whose path is specified via `config.spoon.message_tunnel_file`,
    // and this function reads it and posts the content as a comment, removing the file after that.
    pub fn process_message_tunnel(&self, config: &Config) -> Result<(), Box<dyn Error>> {
        let p = Path::new(&config.spoon.message_tunnel_file);
        if (!p.is_file()) {
            return Ok(());
        }
        let s = std::fs::read_to_string(p)?.trim().to_string();
        std::fs::remove_file(p)?;
        if (!s.is_empty()) {
            self.post_comment(&s)?;
        }
        Ok(())
    }
}
