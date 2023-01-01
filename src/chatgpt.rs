use super::config::Config;

use std::env;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};

pub struct ChatGPT {
    enabled: bool,
    stdin: Option<BufWriter<ChildStdin>>,
    stdout: Option<BufReader<ChildStdout>>,
}

impl ChatGPT {
    pub fn new(config: &Config) -> Self {
        if (!config.chatgpt.enabled) {
            Self {
                enabled: config.chatgpt.enabled,
                stdin: None,
                stdout: None,
            }
        } else {
            env::set_current_dir(&config.chatgpt.project_dir).unwrap();
            let child = Command::new("cargo")
                .args(["run"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            Self {
                enabled: config.chatgpt.enabled,
                stdin: Some(BufWriter::new(child.stdin.unwrap())),
                stdout: Some(BufReader::new(child.stdout.unwrap())),
            }
        }
    }

    pub fn complete(&mut self, prompt: &str) -> Option<String> {
        if (self.enabled) {
            self.stdin
                .as_mut()
                .unwrap()
                .write_all(format!("{}\n", prompt).as_bytes())
                .unwrap();
            self.stdin.as_mut().unwrap().flush().unwrap();
            let mut buf = String::new();
            self.stdout.as_mut().unwrap().read_line(&mut buf).unwrap();
            Some(buf)
        } else {
            None
        }
    }
}
