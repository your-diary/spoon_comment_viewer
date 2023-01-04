use std::process::{Child, Command, Stdio};

use log::error;

/*-------------------------------------*/

#[derive(Default)]
pub struct AudioEffect {
    pub reverb: bool,
    pub high: bool,
    pub low: bool,
    pub left: bool,
    pub right: bool,
    pub fast: bool,
    pub slow: bool,
    pub repeat: bool,

    pub pitch_for_english: bool,
}

/*-------------------------------------*/

pub struct Audio {
    path: String,
    volume: f64,
    effect: AudioEffect,
}

impl Audio {
    pub fn new(path: &str, volume: f64, effect: AudioEffect) -> Self {
        Self {
            path: path.to_string(),
            volume,
            effect,
        }
    }
}

/*-------------------------------------*/

pub struct Player {
    children: Vec<Child>,
}

impl Player {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { children: vec![] }
    }

    //plays the specified audio asynchronically
    //This method returns right away.
    //Playing will stop when `Player.drop()` is called.
    pub fn play_async(&mut self, audio: &Audio) {
        self.play(audio, true);
    }

    pub fn play_sync(&mut self, audio: &Audio) {
        self.play(audio, false);
    }

    fn play(&mut self, audio: &Audio, is_async: bool) {
        let mut args = vec![format!("-v {}", audio.volume), audio.path.clone()];

        //applies audio effects
        {
            let mut set_args = |v: Vec<&'static str>| {
                v.iter().for_each(|e| args.push(e.to_string()));
            };
            if (audio.effect.reverb) {
                set_args(vec!["pad", "0", "2", "reverb"]);
            }
            if (audio.effect.pitch_for_english) {
                if (audio.effect.high) {
                    set_args(vec!["pitch", "550"]);
                } else if (audio.effect.low) {
                    set_args(vec!["pitch", "-300"]);
                } else {
                    set_args(vec!["pitch", "250"]);
                }
            } else {
                if (audio.effect.high) {
                    set_args(vec!["pitch", "300"]);
                }
                if (audio.effect.low) {
                    set_args(vec!["pitch", "-250"]);
                }
            }
            if (audio.effect.left) {
                set_args(vec!["remix", "1v1", "1v0"]);
            }
            if (audio.effect.right) {
                set_args(vec!["remix", "1v0", "1v1"]);
            }
            if (audio.effect.fast) {
                set_args(vec!["tempo", "1.5"]);
            }
            if (audio.effect.slow) {
                set_args(vec!["tempo", "0.6"]);
            }
            if (audio.effect.repeat) {
                set_args(vec!["repeat", "-"]);
            }
        }

        if let Ok(mut c) = Command::new("play")
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if (is_async) {
                self.children.push(c);
            } else {
                #[allow(clippy::collapsible_else_if)]
                if let Err(e) = c.wait() {
                    error!("Failed to play the audio: {}", e);
                }
            }
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.children.iter_mut().for_each(|e| {
            let _ = e.kill();
        });
    }
}

/*-------------------------------------*/
