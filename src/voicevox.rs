use std::{
    collections::HashMap,
    fs,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, SystemTime},
};

use log::error;
use reqwest::{
    blocking::{Client, Response},
    StatusCode,
};

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
                error!("Failed to send the request: {}", e);
                continue;
            }
            Ok(r) => r,
        };

        let response_status = res.status();
        let response_header = res.headers().clone();

        if (!response_status.is_success()) {
            match response_status {
                StatusCode::TOO_MANY_REQUESTS => {
                    error!("`429 Too Many Requests` is returned from VOICEVOX API. Suspended for 10 seconds.");
                    thread::sleep(Duration::from_millis(10000));
                    while (rx.try_recv().is_ok()) {
                        //discards
                    }
                }
                StatusCode::FORBIDDEN => {
                    error!("`403 Forbidden` is returned from VOICEVOX API. This may be temporary.");
                }
                _ => {
                    let body = res.text().unwrap_or_default();
                    if (body.contains("notEnoughPoints")) {
                        error!(
                            "`notEnoughPoints` is returned from VOICEVOX API. Thread terminated."
                        );
                        return;
                    } else {
                        error!("Unknown error is returned from VOICEVOX API: {{ status: {}, body: {} }}", response_status, body);
                    }
                }
            }
            continue;
        }

        let body = match res.bytes() {
            Ok(r) => {
                if (r.is_empty()) {
                    error!("Response from VOICEVOX API is unexpectedly empty.");
                    error!("response header: {:?}", response_header);
                    continue;
                } else {
                    r
                }
            }
            Err(e) => {
                error!("Failed to read response from VOICEVOX API: {}", e);
                error!("response header: {:?}", response_header);
                continue;
            }
        };

        let filepath = format!(
            "{}/{}.wav",
            config.output_dir,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros()
        );
        if let Err(e) = fs::write(&filepath, body) {
            error!("Failed to write to the file [ {} ]: {}", filepath, e);
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
            error!("{}", e);
            self.enabled = false;
        }
    }
}

/*-------------------------------------*/
