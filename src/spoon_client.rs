use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use chrono::Local;
use itertools::Itertools;
use log::error;
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use thirtyfour_sync::error::WebDriverError;

use super::bgm::BGM;
use super::chatgpt::ChatGPT;
use super::comment::CommentType;
use super::config::Config;
use super::constant;
use super::database::{Database, ListenerEntity};
use super::filter::Filter;
use super::listener::Listener;
use super::player::Audio;
use super::player::AudioEffect;
use super::selenium::Selenium;
use super::spoon_core::Spoon;
use super::util;
use super::voicevox::Script;
use super::voicevox::VoiceVox;

/*-------------------------------------*/

struct Logger {
    z: Rc<Selenium>,

    timestamp: String,
    ranking: String,
    num_spoon: String,
    num_heart: String,
    num_current_listener: String,
    num_total_listener: String,
}

impl Logger {
    fn new(z: Rc<Selenium>) -> Self {
        Self {
            z,

            timestamp: String::new(),
            ranking: String::new(),
            num_spoon: String::new(),
            num_heart: String::new(),
            num_current_listener: String::new(),
            num_total_listener: String::new(),
        }
    }

    fn refresh(&mut self) -> Result<(), Box<dyn Error>> {
        self.timestamp = self
            .z
            .inner_text(".time-chip-container span")?
            .trim()
            .to_string();
        if (self.timestamp.len() == 5) {
            self.timestamp = format!("00:{}", self.timestamp);
        }

        let count_info_list = self.z.query_all("ul.count-info-list li")?;
        let mut count_info_list_str = vec![];
        for e in count_info_list {
            count_info_list_str.push(e.text()?.trim().to_string());
        }
        match count_info_list_str.len() {
            //followers-only stream (ranking is not shown)
            4 => {
                count_info_list_str.insert(0, "?".to_string());
            }
            //normal streaming
            5 => {
                //do nothing
            }
            _ => {
                error!(
                    "`count_info_list` is of an unexpected form. Its length is {}.",
                    count_info_list_str.len()
                );
                for _ in 0..(5 - count_info_list_str.len()) {
                    count_info_list_str.insert(0, "?".to_string());
                }
            }
        }
        (
            self.ranking,
            self.num_spoon,
            self.num_heart,
            self.num_current_listener,
            self.num_total_listener,
        ) = (
            count_info_list_str[0].clone(),
            count_info_list_str[1].clone(),
            count_info_list_str[2].clone(),
            count_info_list_str[3].clone(),
            count_info_list_str[4].clone(),
        );

        Ok(())
    }

    fn log(&mut self, color: Option<&str>, s: &str) -> Result<(), Box<dyn Error>> {
        self.refresh()?;

        println!(
            "{}[{} ({}) ({}/{}/{}/{}/{})]{}{} {}{}",
            constant::COLOR_BLACK,
            Local::now().format("%H:%M:%S"),
            self.timestamp,
            self.ranking,
            self.num_spoon,
            self.num_heart,
            self.num_current_listener,
            self.num_total_listener,
            constant::NO_COLOR,
            color.unwrap_or_default(),
            s.replace('\n', "\\n"), //makes it a single line
            if (color.is_some()) {
                constant::NO_COLOR
            } else {
                ""
            }
        );

        Ok(())
    }
}

/*-------------------------------------*/

pub struct SpoonClient {
    spoon: Spoon,

    config: Rc<Config>,

    logger: Logger,

    rng: ThreadRng,

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

            config,

            logger: Logger::new(z.clone()),

            rng: rand::thread_rng(),
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
        self.spoon.update_live_id()
    }

    fn process_message_comment(&mut self, user: &str, text: &str) -> Result<bool, Box<dyn Error>> {
        if (user == self.config.chatgpt.excluded_user) {
            return Ok(true);
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
                return Ok(false);
            }
            let audio_list = &self.config.spoon.live.bgm.audio_list[1..];
            let bgm = audio_list.choose(&mut self.rng).unwrap();
            let audio = Audio::new(&bgm.path, bgm.volume, AudioEffect::default());
            self.bgm.push(&audio);
            let s = format!("再生予定のBGMリストに [ {} ] を追加しました。", bgm.title);
            self.spoon.post_comment(&s)?;
            if (self.config.voicevox.enabled) {
                self.voicevox
                    .say(Script::new(&s, AudioEffect::default(), speaker));
            }
        } else if (self.config.chatgpt.enabled) {
            if (tokens[0] == "help") {
                let s = "help ではなくスラッシュを先頭に付けて\n/help と打ってみてね。";
                self.spoon.post_comment(s)?;
                return Ok(false);
            } else if (tokens[0] == "/help") {
                let s = "[💡ヘルプ]\necho, asmr, zundamon のどれかを\n「/echo　こんにちは」\nのように使ってみてね。\n\n「/bgm」でBGMを変更できるよ。";
                self.spoon.post_comment(s)?;
                return Ok(false);
            } else if (tokens[0].starts_with('/')) {
                match tokens[0] {
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
                        let s = if (tokens[0].is_ascii()) {
                            format!("`{}`は無効なコマンドだよ。`/help`で確認してね。", tokens[0])
                        } else {
                            format!(
                                                "`{}`は無効なコマンドだよ。「/echo　こんにちは」というように、あいだにスペースが入っているか確認してみてね。",
                                                tokens[0]
                                            )
                        };
                        self.spoon.post_comment(&s)?;
                        return Ok(false);
                    }
                }
                if (tokens.len() == 1) {
                    let s = format!(
                        "`{}`単体では使用できないよ。`/help`で確認してね。",
                        tokens[0]
                    );
                    self.spoon.post_comment(&s)?;
                    if (self.config.voicevox.enabled) {
                        self.voicevox
                            .say(Script::new(&s, AudioEffect::default(), speaker));
                    }
                    return Ok(false);
                }
                tokens.remove(0);
                comment_text = tokens.join(" ");
            }

            self.chatgpt.push(Script::new(
                &comment_text.split_whitespace().join(" "),
                effect,
                speaker,
            ));
        }

        Ok(true)
    }

    fn process_guide_comment(&mut self, text: &str) -> Result<(), Box<dyn Error>> {
        let c = text.replace("分前だよ！", "分前だよ");
        self.logger.log(Some(constant::COLOR_WHITE), &c)?;
        if ((c.contains("10分前だよ") || c.contains("5分前だよ") || c.contains("1分前だよ"))
            && self.config.spoon.should_comment_guide)
        {
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

    fn process_like_comment(&mut self, user: &str) -> Result<(), Box<dyn Error>> {
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

    fn process_present_comment(&mut self, user: &str, text: &str) -> Result<(), Box<dyn Error>> {
        let (color, present_name) = if (text.starts_with("ハート")) {
            (constant::COLOR_RED, "バスター")
        } else if (text.starts_with("心ばかりの粗品")) {
            (constant::COLOR_RED, "粗品")
        } else {
            (constant::COLOR_CYAN, "スプーン")
        };

        self.logger.log(
            None,
            &format!("{}{}:{} {}", color, user, constant::NO_COLOR, text),
        )?;

        if (self.config.spoon.should_comment_spoon) {
            let s = format!("{}さん、{}ありがとう。", user, present_name);
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

    fn process_block_comment(&mut self, user: &str) -> Result<(), Box<dyn Error>> {
        let c = format!("{}さんが強制退室になったよ。", user);
        self.logger.log(Some(constant::COLOR_RED), &c)?;
        if (self.config.spoon.should_comment_block) {
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
        if (self.config.chatgpt.enabled) {
            for e in self.chatgpt.fetch() {
                let s = e.script.trim();
                if (s == "QUOTA_ERROR") {
                    let s = "AI部分にエラーが発生しました。管理人に通知を送信しました。一分後、枠を終了します。申し訳ございません。";
                    self.spoon.post_comment(s)?;
                    if (self.config.voicevox.enabled) {
                        self.voicevox.say(Script::new(s, e.effect, e.speaker));
                    }
                    thread::sleep(Duration::from_secs(60));
                    let _ = self.z.close();
                    thread::sleep(Duration::from_secs(60 * 60 * 24 * 31));
                } else {
                    self.spoon.post_comment(s)?;
                    if (self.config.voicevox.enabled) {
                        self.voicevox.say(Script::new(s, e.effect, e.speaker));
                    }
                }
            }
        }

        let comments = self.spoon.retrieve_new_comments()?;

        if (comments.is_empty()) {
            return Ok(());
        }

        //With a small enough check interval, it is unexpected `num_new_comment` has a large value.
        //However, it sometimes happened for some reason: at that time, it seemed the already processed comments in the past were mistakenly treated as new comments.
        //The cause is unknown but we suspect `element_id` may be reassigned by a bug of Spoon or Selenium.
        if (comments.len() >= 15) {
            error!(
                "The value of `num_new_comment` is too large: {}. Ignoring them...",
                comments.len()
            );
            return Ok(());
        }

        for e in comments {
            let user = e.user();
            let text = e.text();

            match e.comment_type() {
                CommentType::Message | CommentType::Combo => {
                    self.logger.log(None, &e.to_string())?;
                    if (!self.process_message_comment(user.unwrap(), text.unwrap())?) {
                        continue;
                    }
                }

                CommentType::Guide => {
                    self.process_guide_comment(text.unwrap())?;

                    //点呼
                    if (text.unwrap().contains("1分前だよ") && self.config.spoon.should_call_over)
                    {
                        self.call_over()?;
                    }
                }

                CommentType::Like => {
                    self.process_like_comment(user.unwrap())?;
                }

                CommentType::Present => {
                    self.process_present_comment(user.unwrap(), text.unwrap())?;
                }

                CommentType::Block => {
                    self.process_block_comment(user.unwrap())?;
                }

                CommentType::Unknown => continue,
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
            if (self.cumulative_listeners.contains(&e)) {
                let entity = self.database.select_by_id(e.id).unwrap();
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
                        entity.visit_count += 1;
                        self.database.update(entity);

                        let ranking = get_ranking();

                        format!(
                            "訪問回数: {}回 / 滞在時間: {} / ランキング: {}位/{}人中",
                            entity.visit_count,
                            util::pretty_print_duration(entity.stay_duration),
                            ranking.0,
                            ranking.1,
                        )
                    } else {
                        let entity = ListenerEntity::new(e.id, 1, Duration::default());
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
