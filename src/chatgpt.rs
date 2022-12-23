use super::config::Config;

use std::env;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::{ChildStdin, Command, Stdio};

pub struct ChatGPT {
    enabled: bool,
    stdin: Option<BufWriter<ChildStdin>>,
}

impl ChatGPT {
    pub fn new(config: &Config) -> Self {
        if (!config.chatgpt_enabled()) {
            Self {
                enabled: config.chatgpt_enabled(),
                stdin: None,
            }
        } else {
            env::set_current_dir(Path::new(&config.chatgpt_binary_path()).parent().unwrap())
                .unwrap();
            let child = Command::new(&config.chatgpt_binary_path())
                .stdin(Stdio::piped())
                .spawn()
                .unwrap();
            Self {
                enabled: config.chatgpt_enabled(),
                stdin: Some(BufWriter::new(child.stdin.unwrap())),
            }
        }
    }

    pub fn complete_and_say(&mut self, prompt: &str) {
        if (self.enabled) {
            self.stdin
                .as_mut()
                .unwrap()
                .write_all(format!("{}\n", prompt).as_bytes())
                .unwrap();
            self.stdin.as_mut().unwrap().flush().unwrap();
        }
    }
}
