use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;

use super::player::Audio;
use super::player::Player;

fn bgm_thread(rx: Receiver<Audio>, audio: Audio) {
    let mut player = Player::new();
    player.play_async(&audio);
    loop {
        let audio: Audio = rx.recv().unwrap();
        player.pause();
        player.play_sync(&audio);
        player.unpause();
    }
}

pub struct BGM {
    tx: Option<SyncSender<Audio>>,
}

impl BGM {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { tx: None }
    }

    pub fn start(&mut self, audio: &Audio) {
        assert!(self.tx.is_none());
        let (tx, rx) = mpsc::sync_channel(0);
        let audio: Audio = audio.clone();
        thread::spawn(move || bgm_thread(rx, audio));
        self.tx = Some(tx);
    }

    pub fn push(&self, audio: &Audio) -> bool {
        if (self.tx.is_none()) {
            return false;
        }
        self.tx.as_ref().unwrap().try_send(audio.clone()).is_ok()
    }
}
