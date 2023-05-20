use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use itertools::Itertools;
use log::error;
use log::info;
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use rand::Rng;
use regex::Regex;
use serde_json::Map;
use serde_json::Value;
use thirtyfour_sync::error::WebDriverError;

use super::bgm::BGM;
use super::chatgpt::ChatGPT;
use super::config::Config;
use super::constant;
use super::database::{Database, ListenerEntity};
use super::filter::Filter;
use super::listener::Listener;
use super::logger::Logger;
use super::models::*;
use super::player::Audio;
use super::player::AudioEffect;
use super::selenium::Selenium;
use super::spoon_core::Spoon;
use super::util;
use super::voicevox::Script;
use super::voicevox::VoiceVox;
use super::websocket::WebSocket;

pub struct SpoonClient {
    spoon: Spoon,
    websocket: WebSocket,

    config: Rc<Config>,

    logger: Logger,

    rng: ThreadRng,

    elapsed: Instant,
    guide_flags: Vec<bool>,

    database: Database,

    chatgpt: ChatGPT,
    voicevox: VoiceVox,
    bgm: BGM,

    z: Rc<Selenium>,

    //listeners
    previous_listeners_set: HashSet<Listener>, //for `いらっしゃい`, `おかえりなさい`, `またきてね`
    previous_listeners_map: HashMap<Listener, Instant>, //for `xxx秒の滞在でした`
    cumulative_listeners: HashSet<Listener>,   //for `おかえりなさい`
}

impl SpoonClient {
    pub fn new(config: Rc<Config>) -> Self {
        let filter = Filter::new(&config.forbidden_words);

        let database = Database::new(Some(&config.database_file));

        let chatgpt = ChatGPT::new(&config, filter.clone());
        let voicevox = VoiceVox::new(&config, filter);
        let bgm = BGM::new();

        let z = Rc::new(Selenium::new(
            config.selenium.webdriver_port,
            Duration::from_millis(config.selenium.implicit_timeout_ms),
            config.selenium.should_maximize_window,
        ));

        Self {
            spoon: Spoon::new(z.clone(), Duration::from_millis(3000)),
            websocket: WebSocket::new(),

            config,

            logger: Logger::new(z.clone()),

            rng: rand::thread_rng(),

            elapsed: Instant::now(),
            guide_flags: vec![false; 3],

            database,
            chatgpt,
            voicevox,
            bgm,
            z,

            previous_listeners_set: HashSet::new(),
            previous_listeners_map: HashMap::new(),
            cumulative_listeners: HashSet::new(),
        }
    }

    pub fn login(
        &self,
        url: &str,
        twitter_id: &str,
        twitter_password: &str,
    ) -> Result<(), WebDriverError> {
        self.spoon.login(url, twitter_id, twitter_password)
    }

    pub fn start_live(&mut self, config: &Config) -> Result<(), Box<dyn Error>> {
        let live = &config.spoon.live;

        if (!live.enabled) {
            return Ok(());
        }

        self.spoon.prepare_live(
            &live.start_url,
            &live.genre,
            &live.title,
            &live.tags,
            &live.pinned_comment,
            if (live.bg_image.is_empty()) {
                None
            } else {
                Some(&live.bg_image)
            },
        )?;

        //bgm
        if (live.bgm.enabled) {
            self.bgm.start(&Audio::new(
                &live.bgm.audio_list[0].path,
                live.bgm.audio_list[0].volume,
                AudioEffect {
                    repeat: true,
                    ..Default::default()
                },
            ));
        }

        if (live.autostart) {
            self.spoon.start_live()?;
        } else {
            print!("Press ENTER after you have started a live: ");
            io::stdout().flush().unwrap();
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            if (buf.trim() == "q") {
                return Err("aborted".into());
            }
        }

        Ok(())
    }

    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        let live_id = self.spoon.update_live_id()?;
        self.websocket.connect(live_id)?;
        self.elapsed = Instant::now();
        Ok(())
    }

    fn process_message_comment(&mut self, o: LiveMessage) -> Result<(), Box<dyn Error>> {
        let text = &o.update_component.message.value;
        let user = &o.data.user.nickname;

        self.logger.log(
            None,
            &format!(
                "{}{}:{} {}",
                constant::COLOR_PURPLE,
                user,
                constant::NO_COLOR,
                text,
            ),
        )?;

        if (user == &self.config.chatgpt.excluded_user) {
            return Ok(());
        }

        let mut comment_text = text.to_string();
        let mut effect = AudioEffect::default();
        let mut speaker = self.config.voicevox.speaker;

        let mut tokens = comment_text.split_whitespace().collect_vec();
        if (tokens.is_empty()) {
            //This happened once.
            return Err("empty comment is unexpectedly detected".into());
        }
        if (tokens[0] == "/bgm") {
            if (self.config.spoon.live.bgm.audio_list.len() <= 1) {
                let s = "BGMの再生に失敗しました。";
                self.spoon.post_comment(s)?;
                if (self.config.voicevox.enabled) {
                    self.voicevox
                        .say(Script::new(s, AudioEffect::default(), speaker));
                }
                return Ok(());
            }
            let audio_list = &self.config.spoon.live.bgm.audio_list[1..];
            let bgm = audio_list.choose(&mut self.rng).unwrap();
            let audio = Audio::new(&bgm.path, bgm.volume, AudioEffect::default());
            self.bgm.push(&audio);
            let s = format!("再生予定のBGMリストに [ {} ] を追加しました。", bgm.title);
            self.spoon.post_comment(&s)?;
            if (self.config.voicevox.enabled) {
                self.voicevox.say(Script::new(
                    "再生予定のBGMリストに楽曲を追加しました",
                    AudioEffect::default(),
                    speaker,
                ));
            }
        } else if (self.config.chatgpt.enabled) {
            if (tokens[0] == "help") {
                let s = "help ではなくスラッシュを先頭に付けて\n/help と打ってみてね。";
                self.spoon.post_comment(s)?;
                return Ok(());
            } else if (tokens[0] == "/help") {
                let s = "[💡ヘルプ]\necho, asmr, zundamon のどれかを\n「/echo　こんにちは」\nのように使ってみてね。\n\n「/bgm」でBGMを変更できるよ。";
                self.spoon.post_comment(s)?;
                return Ok(());
            } else if (tokens[0] == "/fortune") {
                let fortune_names = vec![
                    "総合運",
                    "恋愛運",
                    "金運",
                    "ラッキーナンバー",
                    "ラッキーカラー",
                ];
                let colors = vec![
                    "藍", "青", "青緑", "青紫", "赤", "茜", "小豆", "黄", "黄緑", "金", "銀", "銅",
                    "栗", "黒", "焦茶", "小麦", "紺", "桜", "珊瑚", "漆黒", "朱", "白", "空", "橙",
                    "玉虫", "茶", "灰", "肌", "薔薇", "深緑", "水", "緑", "紫", "桃", "瑠璃",
                    "透明",
                ];

                let mut l = (0..3)
                    .map(|_| "★".repeat(self.rng.gen_range(1..=5)))
                    .collect_vec();
                l.push(self.rng.gen_range(0..=1000).to_string());
                l.push(colors.iter().choose(&mut self.rng).unwrap().to_string());

                let s = format!(
                    "🔮 {}さん\n{}",
                    user,
                    fortune_names
                        .iter()
                        .zip(l)
                        .map(|(name, value)| format!("{}: {}", name, value))
                        .join("\n")
                );
                self.spoon.post_comment(&s)?;
                return Ok(());
            } else if (tokens[0] == "/rank") {
                let ids = self
                    .previous_listeners_map
                    .iter()
                    .filter(|(k, _)| k.nickname == *user)
                    .collect_vec();
                if (ids.len() != 1) {
                    self.spoon
                        .post_comment(&format!("{}さんのランキングの取得に失敗しました", user))?;
                    return Ok(());
                }
                let id = ids[0].0.id;
                let elapsed = ids[0].1.elapsed();

                let all_entities = self
                    .database
                    .select_all()
                    .into_iter()
                    .sorted_by_key(|e| (e.stay_duration, e.visit_count))
                    .rev()
                    .collect_vec();

                let index = all_entities
                    .iter()
                    .position(|entity| entity.id == id)
                    .unwrap();

                let s = format!(
                    "👑 {}さん\nランキング: {}位/{}人中\n滞在時間: {}\n訪問回数: {}回",
                    user,
                    index + 1,
                    all_entities.len(),
                    util::pretty_print_duration(all_entities[index].stay_duration + elapsed),
                    all_entities[index].visit_count,
                );
                self.spoon.post_comment(&s)?;
                return Ok(());
            } else if (tokens[0] == "/ranking") {
                let ranker = self
                    .database
                    .select_all()
                    .into_iter()
                    .sorted_by_key(|e| (e.stay_duration, e.visit_count))
                    .rev()
                    .take(5)
                    .collect_vec();

                let re = Regex::new(r#"\d+秒"#).unwrap();

                let s = format!(
                    "👑 ランキング\n{}",
                    ranker
                        .into_iter()
                        .enumerate()
                        .map(|(i, e)| format!(
                            "{}. {}({}回)",
                            i + 1,
                            re.replace(&util::pretty_print_duration(e.stay_duration), ""),
                            e.visit_count
                        ))
                        .join("\n")
                );
                self.spoon.post_comment(&s)?;
                return Ok(());
            } else if (tokens[0].starts_with('/')) {
                let mut num_command = 0;
                for token in &tokens {
                    if (!token.starts_with('/')) {
                        break;
                    }
                    num_command += 1;
                    match *token {
                        "/reverb" => effect.reverb = true,
                        "/echo" => effect.reverb = true, //same as `/reverb`
                        "/high" => effect.high = true,
                        "/low" => effect.low = true,
                        "/left" => effect.left = true, //low quality on Linux
                        "/right" => effect.right = true, //low quality on Linux
                        "/fast" => effect.fast = true,
                        "/slow" => effect.slow = true,

                        "/zundamon" => speaker = 3,
                        "/zundamon_2" => speaker = 1,
                        "/zundamon_3" => speaker = 7,
                        "/zundamon_4" => speaker = 5,
                        "/zundamon_5" => speaker = 38,

                        "/asmr" => speaker = 22,

                        "/sayo" => speaker = 46,

                        "/tsumugi" => speaker = 8,

                        "/himari" => speaker = 14,

                        "/nurse" => speaker = 47,
                        "/nurse_asmr" => speaker = 50,

                        "/bii" => speaker = 58,
                        "/bii_calm" => speaker = 59,
                        "/bii_shy" => speaker = 60,

                        _ => {
                            let s = if (token.is_ascii()) {
                                format!("`{}`は無効なコマンドだよ。`/help`で確認してね。", token,)
                            } else {
                                format!(
                                                    "`{}`は無効なコマンドだよ。「/echo　こんにちは」というように、あいだにスペースが入っているか確認してみてね。",
                                                    token,
                                                )
                            };
                            self.spoon.post_comment(&s)?;
                            return Ok(());
                        }
                    }
                }
                if (tokens.len() == num_command) {
                    let command = tokens.iter().take(num_command).join(" ");
                    let s = format!(
                        "`{}`単体では使用できないよ。「{}　こんにちは」のように、テキストを足してみてね。",
                        command,
                        command,
                    );
                    self.spoon.post_comment(&s)?;
                    return Ok(());
                }
                for _ in 0..num_command {
                    tokens.remove(0);
                }
                comment_text = tokens.join(" ");
            }

            self.chatgpt.push(Script::new(
                &comment_text.split_whitespace().join(" "),
                effect,
                speaker,
            ));
        }

        Ok(())
    }

    fn process_guide(&mut self) -> Result<(), Box<dyn Error>> {
        let elapsed = self.elapsed.elapsed();

        let guide_10 = Duration::from_secs(3600 * 2 - 10 * 60);
        let guide_5 = Duration::from_secs(3600 * 2 - 5 * 60);
        let guide_1 = Duration::from_secs(3600 * 2 - 60);

        let mut should_call_over = self.config.spoon.should_call_over;

        let message = if ((elapsed > guide_10) && !self.guide_flags[0]) {
            self.guide_flags[0] = true;
            "配信終了10分前だよ"
        } else if ((elapsed > guide_5) && !self.guide_flags[1]) {
            self.guide_flags[1] = true;
            "配信終了5分前だよ"
        } else if ((elapsed > guide_1) && !self.guide_flags[2]) {
            self.guide_flags[2] = true;
            should_call_over &= true;
            "配信終了1分前だよ"
        } else {
            return Ok(());
        };

        self.logger.log(Some(constant::COLOR_WHITE), message)?;
        if (self.config.spoon.should_comment_guide) {
            self.spoon.post_comment(message)?;
            if (self.config.voicevox.enabled) {
                self.voicevox.say(Script::new(
                    message,
                    AudioEffect::default(),
                    self.config.voicevox.speaker,
                ));
            }
        }

        if (should_call_over) {
            self.call_over()?;
        }

        Ok(())
    }

    fn process_like_comment(&mut self, o: LiveLike) -> Result<(), Box<dyn Error>> {
        let user = o.data.author.nickname;
        let c = format!("{}さん、ハートありがとう。", user);
        self.logger.log(Some(constant::COLOR_YELLOW), &c)?;
        if (self.config.spoon.should_comment_heart) {
            self.spoon.post_comment(&c)?;
            if (self.config.voicevox.enabled) {
                self.voicevox.say(Script::new(
                    &c,
                    AudioEffect {
                        reverb: true,
                        ..Default::default()
                    },
                    self.config.voicevox.speaker,
                ));
            }
        }
        Ok(())
    }

    fn process_use_item_comment(&mut self, o: UseItem) -> Result<(), Box<dyn Error>> {
        let user = o.data.user.nickname;
        let (item_id, effect, amount) = match o.use_items.get(0) {
            None => return Err("`use_items` is empty".into()),
            Some(l) => (l.item_id, &l.effect, l.amount),
        };

        let item_name = if (item_id == 34) {
            "粗品"
        } else if (effect == "LIKE") {
            "バスター"
        } else {
            "謎のアイテム"
        };

        self.logger.log(
            None,
            &format!(
                "{}{}:{} {} {}",
                constant::COLOR_RED,
                user,
                constant::NO_COLOR,
                amount,
                item_name
            ),
        )?;

        if (self.config.spoon.should_comment_spoon) {
            let s = format!("{}さん、{}ありがとう。", user, item_name);
            self.spoon.post_comment(&s)?;
            if (self.config.voicevox.enabled) {
                self.voicevox.say(Script::new(
                    &s,
                    AudioEffect {
                        reverb: true,
                        ..Default::default()
                    },
                    self.config.voicevox.speaker,
                ));
            }
        }

        Ok(())
    }

    fn process_present_like_comment(&mut self, o: LivePresentLike) -> Result<(), Box<dyn Error>> {
        let user = o.data.user.nickname;
        let amount = o.update_component.like.amount * o.update_component.like.combo;

        self.logger.log(
            None,
            &format!(
                "{}{}:{} {} ハート",
                constant::COLOR_RED,
                user,
                constant::NO_COLOR,
                amount
            ),
        )?;

        if (self.config.spoon.should_comment_spoon) {
            let s = format!("{}さん、バスターありがとう。", user);
            self.spoon.post_comment(&s)?;
            if (self.config.voicevox.enabled) {
                self.voicevox.say(Script::new(
                    &s,
                    AudioEffect {
                        reverb: true,
                        ..Default::default()
                    },
                    self.config.voicevox.speaker,
                ));
            }
        }

        Ok(())
    }

    fn process_present_comment(&mut self, o: LivePresent) -> Result<(), Box<dyn Error>> {
        let user = o.data.author.nickname;
        let amount = o.data.amount * o.data.combo;

        self.logger.log(
            None,
            &format!(
                "{}{}:{} {} Spoon",
                constant::COLOR_CYAN,
                user,
                constant::NO_COLOR,
                amount
            ),
        )?;

        if (self.config.spoon.should_comment_spoon) {
            let s = format!("{}さん、スプーンありがとう。", user);
            self.spoon.post_comment(&s)?;
            if (self.config.voicevox.enabled) {
                self.voicevox.say(Script::new(
                    &s,
                    AudioEffect {
                        reverb: true,
                        ..Default::default()
                    },
                    self.config.voicevox.speaker,
                ));
            }
        }

        Ok(())
    }

    //点呼
    fn call_over(&mut self) -> Result<(), Box<dyn Error>> {
        let c = "点呼するよ。";
        self.spoon.post_comment(c)?;
        if (self.config.voicevox.enabled) {
            self.voicevox.say(Script::new(
                c,
                AudioEffect::default(),
                self.config.voicevox.speaker,
            ));
        }
        for listener in &self.previous_listeners_set {
            let c = format!("{}さん、来てくれてありがとう。", listener.nickname);
            self.spoon.post_comment(&c)?;
            if (self.config.voicevox.enabled) {
                self.voicevox.say(Script::new(
                    &c,
                    AudioEffect::default(),
                    self.config.voicevox.speaker,
                ));
            }
        }
        Ok(())
    }

    pub fn process_comments(&mut self) -> Result<(), Box<dyn Error>> {
        self.process_guide().unwrap_or_else(|e| error!("{}", e));

        if (self.config.chatgpt.enabled) {
            for e in self.chatgpt.fetch() {
                let s = e.script.trim();
                if (s == "QUOTA_ERROR") {
                    let _ = Command::new("curl")
                        .args([
                            &self.config.chatgpt.discord_url,
                            "-d",
                            r#"{"wait": true, "content": "OpenAI API quota exceeded"}"#,
                            "-H",
                            "Content-Type: application/json",
                        ])
                        .status();
                    let s = "AI部分にエラーが発生しました。管理人に通知を送信しました。一分後、枠を終了します。申し訳ございません。";
                    let _ = self.spoon.post_comment(s);
                    if (self.config.voicevox.enabled) {
                        self.voicevox.say(Script::new(s, e.effect, e.speaker));
                    }
                    thread::sleep(Duration::from_secs(60));
                    let _ = self.z.close();
                    thread::sleep(Duration::from_secs(60 * 60 * 24 * 31));
                } else {
                    self.spoon
                        .post_comment(s)
                        .unwrap_or_else(|e| error!("{}", e));
                    if (self.config.voicevox.enabled) {
                        self.voicevox.say(Script::new(s, e.effect, e.speaker));
                    }
                }
            }
        }

        let comments = self.websocket.fetch();

        if (comments.is_empty()) {
            return Ok(());
        }

        for s in comments {
            //for performance
            if (s.starts_with(r#"{"event":"live_update","#)
                || s.starts_with(r#"{"event":"live_rank","#))
            {
                continue;
            }

            let m: Map<String, Value> = match serde_json::from_str::<Value>(&s) {
                Ok(v) => match v.as_object() {
                    Some(m) => m.clone(),
                    None => {
                        error!("WebSocket message is not a JSON object: {}", v);
                        continue;
                    }
                },
                Err(e) => {
                    error!("WebSocket message is not JSON: {} in {}", e, s);
                    continue;
                }
            };

            let event_type = match m.get("event") {
                None => {
                    error!("no event field found in object: {}", s);
                    continue;
                }
                Some(Value::String(s)) => s,
                _ => {
                    error!("event field is not a string: {}", s);
                    continue;
                }
            };

            match event_type.as_str() {
                "live_join" => {
                    let o = match serde_json::from_str::<LiveJoin>(&s) {
                        Ok(o) => o,
                        Err(e) => {
                            error!("deserialization error: {} in {}", e, s);
                            continue;
                        }
                    };
                    assert_eq!("success", o.result.detail);
                    info!("WebSocket connection succeeded.");
                }
                "live_rank" => (),
                "live_update" => (),
                //comment
                "live_message" => {
                    let o = match serde_json::from_str::<LiveMessage>(&s) {
                        Ok(o) => o,
                        Err(e) => {
                            error!("deserialization error: {} in {}", e, s);
                            continue;
                        }
                    };
                    self.process_message_comment(o)
                        .unwrap_or_else(|e| error!("{}", e));
                }
                //heart
                "live_like" => {
                    let o = match serde_json::from_str::<LiveLike>(&s) {
                        Ok(o) => o,
                        Err(e) => {
                            error!("deserialization error: {} in {}", e, s);
                            continue;
                        }
                    };
                    self.process_like_comment(o)
                        .unwrap_or_else(|e| error!("{}", e));
                }
                //粗品
                "use_item" => {
                    let o = match serde_json::from_str::<UseItem>(&s) {
                        Ok(o) => o,
                        Err(e) => {
                            error!("deserialization error: {} in {}", e, s);
                            continue;
                        }
                    };
                    self.process_use_item_comment(o)
                        .unwrap_or_else(|e| error!("{}", e));
                }
                //buster
                "live_present_like" => {
                    let o = match serde_json::from_str::<LivePresentLike>(&s) {
                        Ok(o) => o,
                        Err(e) => {
                            error!("deserialization error: {} in {}", e, s);
                            continue;
                        }
                    };
                    self.process_present_like_comment(o)
                        .unwrap_or_else(|e| error!("{}", e));
                }
                //spoon
                "live_present" => {
                    let o = match serde_json::from_str::<LivePresent>(&s) {
                        Ok(o) => o,
                        Err(e) => {
                            error!("deserialization error: {} in {}", e, s);
                            continue;
                        }
                    };
                    self.process_present_comment(o)
                        .unwrap_or_else(|e| error!("{}", e));
                }
                t => {
                    error!("unknown event type: {} in {}", t, s);
                    continue;
                }
            }
        }

        Ok(())
    }

    pub fn process_listeners(&mut self, config: &Config) -> Result<(), Box<dyn Error>> {
        let listeners_set = self
            .spoon
            .retrieve_listeners()?
            .into_iter()
            .collect::<HashSet<_>>();

        let exited_listeners = &self.previous_listeners_set - &listeners_set;
        let new_listeners = &listeners_set - &self.previous_listeners_set;

        for e in exited_listeners {
            let c = format!("{}さん、また来てね。", e.nickname);
            let stay_duration = self.previous_listeners_map.get(&e).unwrap().elapsed();
            let c_with_time = format!(
                "{}(滞在時間: {})",
                c,
                util::pretty_print_duration(stay_duration),
            );
            {
                let mut entity = self.database.select_by_id(e.id).unwrap();
                entity.stay_duration += stay_duration;
                self.database.update(entity);
            }
            self.logger.log(Some(constant::COLOR_GREEN), &c_with_time)?;
            if (config.spoon.should_comment_listener) {
                self.spoon.post_comment(&c_with_time)?;
                if (config.voicevox.enabled) {
                    self.voicevox.say(Script::new(
                        &c,
                        AudioEffect::default(),
                        config.voicevox.speaker,
                    ));
                }
            }
            self.previous_listeners_map.remove(&e);
        }

        for e in new_listeners {
            self.previous_listeners_map
                .insert(e.clone(), Instant::now());

            let get_ranking = || -> (usize, usize) {
                let all_entities = self
                    .database
                    .select_all()
                    .into_iter()
                    .sorted_by_key(|e| (e.stay_duration, e.visit_count))
                    .rev()
                    .collect_vec();

                let ranking = all_entities
                    .iter()
                    .position(|entity| entity.id == e.id)
                    .unwrap()
                    + 1;

                (ranking, all_entities.len())
            };

            //おかえりなさい
            if (self
                .cumulative_listeners
                .iter()
                .any(|listener| listener.id == e.id))
            {
                let entity = self.database.select_by_id(e.id).unwrap();
                if (entity.name != e.nickname) {
                    let mut entity = entity.clone();
                    entity.name = e.nickname.clone();
                    self.database.update(entity);
                }
                let ranking = get_ranking();
                #[allow(clippy::format_in_format_args)]
                let c = format!(
                    "{}さん、おかえりなさい。\n({})",
                    e.nickname,
                    format!(
                        "訪問回数: {}回 / 滞在時間: {} / ランキング: {}位/{}人中",
                        entity.visit_count,
                        util::pretty_print_duration(entity.stay_duration),
                        ranking.0,
                        ranking.1,
                    )
                );
                self.logger.log(Some(constant::COLOR_GREEN), &c)?;
                if (config.spoon.should_comment_listener) {
                    self.spoon.post_comment(&c)?;
                    if (config.voicevox.enabled) {
                        self.voicevox.say(Script::new(
                            c.split('\n').next().unwrap(),
                            AudioEffect::default(),
                            config.voicevox.speaker,
                        ));
                    }
                }

            //いらっしゃい
            } else {
                self.cumulative_listeners.insert(e.clone());
                let c = format!(
                    "{}さん、いらっしゃい。\n({})",
                    e.nickname,
                    if let Some(mut entity) = self.database.select_by_id(e.id) {
                        entity.name = e.nickname.clone();
                        entity.visit_count += 1;
                        self.database.update(entity.clone());

                        let ranking = get_ranking();

                        format!(
                            "訪問回数: {}回 / 滞在時間: {} / ランキング: {}位/{}人中",
                            entity.visit_count,
                            util::pretty_print_duration(entity.stay_duration),
                            ranking.0,
                            ranking.1,
                        )
                    } else {
                        let entity =
                            ListenerEntity::new(e.id, e.nickname.clone(), 1, Duration::default());
                        self.database.insert(entity);

                        let entities = self.database.select_all();

                        format!(
                            "初見さん / ランキング: {}位/{}人中",
                            entities.len(),
                            entities.len()
                        )
                    }
                );
                self.logger.log(
                    Some(constant::COLOR_GREEN),
                    &format!("{} ({:?})", c, e), //We print `e` itself to trace the unique user id of a troll.
                )?;
                if (config.spoon.should_comment_listener) {
                    self.spoon.post_comment(&c)?;
                    if (config.voicevox.enabled) {
                        self.voicevox.say(Script::new(
                            c.split('\n').next().unwrap(),
                            AudioEffect::default(),
                            config.voicevox.speaker,
                        ));
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
    pub fn process_message_tunnel(&mut self) -> Result<(), Box<dyn Error>> {
        let p = Path::new(&self.config.spoon.message_tunnel_file);
        if (!p.is_file()) {
            return Ok(());
        }
        let s = fs::read_to_string(p)?.trim().to_string();
        fs::remove_file(p)?;
        if (!s.is_empty()) {
            self.spoon.post_comment(&format!("(運営より) {}", s))?;
            if (self.config.voicevox.enabled) {
                self.voicevox.say(Script::new(
                    &s,
                    AudioEffect::default(),
                    self.config.voicevox.speaker,
                ));
            }
        }
        Ok(())
    }
}

impl Drop for SpoonClient {
    fn drop(&mut self) {
        for (listener, instant) in &self.previous_listeners_map {
            let mut entity = self.database.select_by_id(listener.id).unwrap();
            entity.stay_duration += instant.elapsed();
            self.database.update(entity);
        }
    }
}
