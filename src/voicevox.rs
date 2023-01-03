use std::{
    collections::HashMap,
    fs,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, SystemTime},
};

use reqwest::blocking::{Client, Response};

use super::config::Config;
use super::player::Audio;
use super::player::AudioEffect;
use super::player::Player;
use super::util;

/*-------------------------------------*/

struct APIRequest {
    script: String,
    effect: AudioEffect,
}

impl APIRequest {
    fn new(script: &str, effect: AudioEffect) -> Self {
        Self {
            script: script.to_string(),
            effect,
        }
    }
}

/*-------------------------------------*/

fn player_thread(rx: Receiver<Audio>) {
    let mut player = Player::new();
    loop {
        let audio: Audio = rx.recv().unwrap();
        player.play_sync(&audio);
    }
}

fn api_thread(rx: Receiver<APIRequest>, config: Config) {
    let config = config.voicevox;

    let (tx, rx2) = mpsc::channel();
    thread::spawn(move || player_thread(rx2));

    let client = Client::builder()
        .timeout(Some(Duration::from_secs(config.timeout_sec)))
        .build()
        .unwrap();

    loop {
        let req: APIRequest = rx.recv().unwrap();

        let mut params = HashMap::new();
        let speed = config.speed.to_string();
        params.insert("key", &config.api_key);
        params.insert("speaker", &config.speaker);
        params.insert("speed", &speed);
        params.insert("text", &req.script);

        let res: Response = match client.get(&config.url).query(&params).send() {
            Err(e) => {
                println!("{}", e);
                continue;
            }
            Ok(r) => r,
        };

        if (!res.status().is_success()) {
            let body = res.text().unwrap_or_default();
            if (body.contains("429")) {
                println!("`429 Too Many Requests` is returned from VOICEVOX API. Suspended for 10 seconds.");
                thread::sleep(Duration::from_millis(10000));
                while (rx.try_recv().is_ok()) {
                    //discards
                }
            } else if (body.contains("notEnoughPoints")) {
                println!("`notEnoughPoints` is returned from VOICEVOX API. Thread terminated.");
                return;
            } else {
                println!(
                    "Ignorable error is returned from VOICEVOX API: [ {} ]",
                    body
                );
            }
            continue;
        }

        let body = match res.bytes() {
            Ok(r) => r,
            Err(e) => {
                println!("Response from VOICEVOX API is unexpectedly empty: {}", e);
                continue;
            }
        };

        let filepath = format!(
            "{}/{}.wav",
            config.output_dir,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        if let Err(e) = fs::write(&filepath, body) {
            println!("Failed to write to the file [ {} ]: {}", filepath, e);
            continue;
        }

        let audio = Audio::new(&filepath, 1., req.effect);
        tx.send(audio).unwrap();
    }
}

/*-------------------------------------*/

pub struct VoiceVox {
    enabled: bool,
    should_skip_non_japanese: bool,
    tx: Option<Sender<APIRequest>>,
}

impl VoiceVox {
    pub fn new(config: &Config) -> Self {
        if (config.voicevox.enabled) {
            let should_skip_non_japanese = config.voicevox.should_skip_non_japanese;
            let (tx, rx) = mpsc::channel();
            let config = config.clone();
            thread::spawn(move || api_thread(rx, config));
            Self {
                enabled: true,
                should_skip_non_japanese,
                tx: Some(tx),
            }
        } else {
            Self {
                enabled: false,
                should_skip_non_japanese: false,
                tx: None,
            }
        }
    }

    pub fn say(&mut self, script: &str, effect: AudioEffect) {
        if (!self.enabled) {
            return;
        }
        if (self.should_skip_non_japanese && !util::is_japanese(script)) {
            return;
        }
        let req = APIRequest::new(script, effect);
        if let Err(e) = self.tx.as_ref().unwrap().send(req) {
            println!("{}", e);
            self.enabled = false;
        }
    }
}

/*-------------------------------------*/
