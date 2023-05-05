use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::{env, thread};

use log::info;

use super::config::Config;
use super::filter::Filter;
use super::voicevox::Script;

fn chatgpt_thread(rx: Receiver<Script>, tx: Sender<Script>, project_dir: String, filter: Filter) {
    env::set_current_dir(&project_dir).unwrap();

    let child = Command::new("cargo")
        .args(["run"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = BufWriter::new(child.stdin.unwrap());
    let mut stdout = BufReader::new(child.stdout.unwrap());

    let mut buf = String::new();
    loop {
        let mut script = rx.recv().unwrap();
        if (!filter.is_normal(&script.script)) {
            let original = script.script.clone();
            let sanitized = filter.sanitize(&script.script);
            info!(
                "Forbidden word sanitized: [{}] -> [{}]",
                original, sanitized
            );
            script.script = sanitized;
        }
        stdin
            .write_all(format!("{}\n", script.script).as_bytes())
            .unwrap();
        stdin.flush().unwrap();

        buf.clear();
        stdout.read_line(&mut buf).unwrap();
        script.script = buf.clone();
        tx.send(script).unwrap();
    }
}

pub struct ChatGPT {
    tx: Option<Sender<Script>>,
    rx: Option<Receiver<Script>>,
}

impl ChatGPT {
    pub fn new(config: &Config, filter: Filter) -> Self {
        if (!config.chatgpt.enabled) {
            Self { tx: None, rx: None }
        } else {
            let (tx, rx) = mpsc::channel();
            let (tx2, rx2) = mpsc::channel();
            let project_dir = config.chatgpt.project_dir.clone();
            thread::spawn(move || chatgpt_thread(rx, tx2, project_dir, filter));
            Self {
                tx: Some(tx),
                rx: Some(rx2),
            }
        }
    }

    pub fn fetch(&self) -> Vec<Script> {
        if (self.rx.is_none()) {
            return vec![];
        }
        let mut ret = vec![];
        while let Ok(s) = self.rx.as_ref().unwrap().try_recv() {
            ret.push(s);
        }
        ret
    }

    pub fn push(&self, script: Script) {
        if (self.tx.is_none()) {
            return;
        }
        self.tx.as_ref().unwrap().send(script).unwrap();
    }
}
