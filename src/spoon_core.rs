use std::{error::Error, path::Path, rc::Rc, time::Duration, time::Instant};

use itertools::Itertools;
use log::error;
use log::info;
use regex::Regex;
use reqwest::blocking::Client;
use thirtyfour_sync::error::WebDriverError;
use thirtyfour_sync::ElementId;

use super::comment::Comment;
use super::comment::CommentType;
use super::listener::{Listener, Listeners};
use super::selenium::Selenium;

pub struct Spoon {
    z: Rc<Selenium>,
    http_client: Client,
    live_id: u64,
    previous_commenter: String, //for combo comment
    existing_comments: Vec<ElementId>,
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
            previous_commenter: String::new(),
            existing_comments: Vec::with_capacity(1000),
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

    pub fn update_live_id(&mut self) -> Result<(), Box<dyn Error>> {
        if let serde_json::value::Value::Number(n) = self.z.execute_javascript(
            "return JSON.parse(window.localStorage.SPOONCAST_liveBroadcastOnair).liveId;",
        )? {
            match n.as_u64() {
                Some(id) => {
                    self.live_id = id;
                    Ok(())
                }
                None => Err("Failed to parse the live id as number.".into()),
            }
        } else {
            Err("Failed to retrieve the live id.".into())
        }
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

    pub fn retrieve_new_comments(&mut self) -> Result<Vec<Comment>, WebDriverError> {
        let mut ret = vec![];

        let start = Instant::now();
        let l = self.z.query_all("li.chat-list-item")?;
        info!("self.z.query_all: {}ms", start.elapsed().as_millis());

        let num_new_comment = if (self.existing_comments.is_empty()) {
            l.len()
        } else if let Some(i) = l
            .iter()
            .rposition(|e| &e.element_id == self.existing_comments.last().unwrap())
        {
            l.len() - (i + 1)
        } else {
            l.len()
        };

        if (num_new_comment == 0) {
            return Ok(vec![]);
        }

        for e in l.iter().skip(l.len() - num_new_comment) {
            let element_id = e.element_id.clone();

            self.existing_comments.push(element_id.clone());

            let inner_text = match e.text() {
                Ok(s) => s,
                Err(_) => continue,
            };

            let class_name = match e.class_name() {
                Err(e) => {
                    error!("{}", e);
                    continue;
                }
                Ok(s) => s,
            };

            let comment_type = CommentType::new(class_name);

            match comment_type {
                CommentType::Message | CommentType::Combo => {
                    let is_combo = comment_type == CommentType::Combo;

                    let comment = if (is_combo) {
                        Comment::new(
                            element_id,
                            comment_type,
                            Some(self.previous_commenter.clone()),
                            Some(inner_text.clone()),
                        )
                    } else {
                        let tokens = inner_text.splitn(2, '\n').collect_vec();
                        if (tokens.len() != 2) {
                            error!("Comment [ {} ] has an unexpected form.", inner_text);
                            continue;
                        }
                        Comment::new(
                            element_id,
                            comment_type,
                            Some(tokens[0].to_string()),
                            Some(tokens[1].to_string()),
                        )
                    };

                    if (!is_combo) {
                        self.previous_commenter = String::from(comment.user().unwrap());
                    }

                    ret.push(comment);
                }

                CommentType::Guide => {
                    ret.push(Comment::new(
                        element_id,
                        comment_type,
                        None,
                        Some(inner_text.clone()),
                    ));
                }

                //`buster` is categorized as `CommentType::Present`
                CommentType::Like => {
                    ret.push(Comment::new(
                        element_id,
                        comment_type,
                        Some(inner_text.replace("さんがハートを押したよ！", "")),
                        None,
                    ));
                }

                //includes `buster`
                CommentType::Present => {
                    let pat = Regex::new(r#"^([^\n]*)\n+(.*Spoon.*|ハート.*|心ばかりの粗品.*)$"#)
                        .unwrap();
                    if let Some(groups) = pat.captures(&inner_text) {
                        let user = groups.get(1).unwrap().as_str().to_string();
                        let text = groups.get(2).unwrap().as_str().to_string();
                        ret.push(Comment::new(
                            element_id,
                            comment_type,
                            Some(user),
                            Some(text),
                        ));
                    }
                }

                CommentType::Block => {
                    ret.push(Comment::new(
                        element_id,
                        comment_type,
                        Some(inner_text.replace("さんが強制退室になりました。", "")),
                        None,
                    ));
                }

                CommentType::Unknown => continue,
            }
        }

        Ok(ret)
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
