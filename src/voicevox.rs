use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs,
    hash::{Hash, Hasher},
    path::Path,
    process::Command,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use log::{error, info};
use reqwest::{
    blocking::{Client, Response},
    StatusCode,
};

use super::config::Config;
use super::filter::Filter;
use super::player::Audio;
use super::player::AudioEffect;
use super::player::Player;
use super::util;

/*-------------------------------------*/

struct APIRequest {
    script: String,
    effect: AudioEffect,
    speaker: usize,
}

impl APIRequest {
    fn new(script: &str, effect: AudioEffect, speaker: usize) -> Self {
        Self {
            script: script.to_string(),
            effect,
            speaker,
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

fn calculate_hash(s: &str) -> String {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish().to_string()
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

        //for English
        if (req.effect.pitch_for_english) {
            let filepath = format!("{}/{}.mp3", config.output_dir, calculate_hash(&req.script));

            if (!Path::new(&filepath).is_file()) {
                let res = match Command::new("google_speech")
                    .args(["--output", &filepath, &req.script])
                    .output()
                {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Failed to execute `google_speech`: {}", e);
                        continue;
                    }
                };

                if (!res.status.success()) {
                    error!(
                        "Non-zero exit status is returned from `google_speech`: {}",
                        String::from_utf8(res.stderr).unwrap_or_default()
                    );
                    continue;
                }
            }

            let audio = Audio::new(&filepath, 2., req.effect);
            tx.send(audio).unwrap();

        //for Japanese
        } else {
            let mut params = HashMap::new();
            let speaker = req.speaker.to_string();
            let speed = config.speed.to_string();
            params.insert("key", &config.api_key);
            params.insert("speaker", &speaker);
            params.insert("speed", &speed);
            params.insert("text", &req.script);

            let filepath = format!(
                "{}/{}.wav",
                config.output_dir,
                calculate_hash(&format!("{}_{}_{}", speaker, speed, req.script))
            );
            if (!Path::new(&filepath).is_file()) {
                let res: Response = match client.get(&config.url).query(&params).send() {
                    Err(e) => {
                        if (e.is_timeout()) {
                            error!("VOICEVOX API timed out.");
                        } else if (e.is_connect()) {
                            error!("Failed to connect to VOICEVOX API.");
                        } else {
                            error!("Failed to send the request: {}", e);
                        }
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
                            error!(
                            "`403 Forbidden` is returned from VOICEVOX API. This may be temporary."
                        );
                        }
                        StatusCode::SERVICE_UNAVAILABLE => {
                            error!(
                            "`503 Service Unavailable` is returned from VOICEVOX API. This may be temporary."
                        );
                        }
                        _ => {
                            let body = res.text().unwrap_or_default();
                            if (body.contains("notEnoughPoints")) {
                                error!("`notEnoughPoints` is returned from VOICEVOX API. Thread terminated.");
                                return;
                            } else if (body.contains(r#""errorMessage": "failed""#)) {
                                error!("`failed` is returned from VOICEVOX API. This is expected to randomly occur.");
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

                if let Err(e) = fs::write(&filepath, body) {
                    error!("Failed to write to the file [ {} ]: {}", filepath, e);
                    continue;
                }
            }

            let audio = Audio::new(&filepath, 1., req.effect);
            tx.send(audio).unwrap();
        }
    }
}

/*-------------------------------------*/

pub struct Script {
    pub script: String,
    pub effect: AudioEffect,
    pub speaker: usize,
}

impl Script {
    pub fn new(script: &str, effect: AudioEffect, speaker: usize) -> Self {
        Self {
            script: script.to_string(),
            effect,
            speaker,
        }
    }
}

pub struct VoiceVox {
    enabled: bool,
    should_skip_non_japanese: bool,
    should_use_google_speech_for_non_japanese: bool,
    tx: Option<Sender<APIRequest>>,
    filter: Option<Filter>,
}

impl VoiceVox {
    pub fn new(config: &Config, filter: Filter) -> Self {
        if (config.voicevox.enabled) {
            let should_skip_non_japanese = config.voicevox.should_skip_non_japanese;
            let should_use_google_speech_for_non_japanese =
                config.voicevox.should_use_google_speech_for_non_japanese;
            let (tx, rx) = mpsc::channel();
            let config = config.clone();
            thread::spawn(move || api_thread(rx, config));
            Self {
                enabled: true,
                should_skip_non_japanese,
                should_use_google_speech_for_non_japanese,
                tx: Some(tx),
                filter: Some(filter),
            }
        } else {
            Self {
                enabled: false,
                should_skip_non_japanese: false,
                should_use_google_speech_for_non_japanese: false,
                tx: None,
                filter: None,
            }
        }
    }

    pub fn say(&mut self, mut script: Script) {
        if (!self.enabled) {
            return;
        }
        if (!self.filter.as_ref().unwrap().is_normal(&script.script)) {
            info!("Forbidden word detected: [{}]", script.script);
            return;
        }
        if (self.should_skip_non_japanese && !util::is_japanese(&script.script)) {
            if (self.should_use_google_speech_for_non_japanese) {
                script.effect.pitch_for_english = true;
            } else {
                return;
            }
        }
        let req = APIRequest::new(&script.script, script.effect, script.speaker);
        if let Err(e) = self.tx.as_ref().unwrap().send(req) {
            error!("{}", e);
            self.enabled = false;
        }
    }
}

/*-------------------------------------*/
