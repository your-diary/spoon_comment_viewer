use std::process::Command;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;

use super::player::Audio;
use super::player::Player;

fn bgm_thread(rx: Receiver<Audio>, audio: Audio) {
    let mut player = Player::new();
    std::thread::sleep(std::time::Duration::from_millis(100)); //for unknown reason, without this, the following `play_async()` silently failed
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

//HACK: This is dirty, but it cannot be helped as `Player::drop()` isn't called in another thread (i.e. `bgm_thread()`).
impl Drop for BGM {
    fn drop(&mut self) {
        let _ = Command::new("killall").args(["play"]).output();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // #[ignore]
    fn test01() {
        let bgm = BGM::new();
        assert_eq!(false, bgm.push(&Audio::new("", 1., Default::default())));
    }

    #[test]
    // #[ignore]
    fn test02() {
        env_logger::init();
        let mut bgm = BGM::new();
        bgm.start(&Audio::new(
            "./test_assets/long.mp3",
            1.,
            Default::default(),
        ));
        std::thread::sleep(std::time::Duration::from_millis(5000));
        assert_eq!(
            true,
            bgm.push(&Audio::new(
                "./test_assets/short.mp3",
                1.,
                Default::default(),
            ))
        );
        assert_eq!(
            false,
            bgm.push(&Audio::new(
                "./test_assets/short.mp3",
                1.,
                Default::default(),
            ))
        );
        std::thread::sleep(std::time::Duration::from_millis(12000));
        std::thread::sleep(std::time::Duration::from_millis(3000));
    }
}
