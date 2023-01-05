use std::env;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::rc::Rc;

use super::config::Config;
use super::filter::Filter;

pub struct ChatGPT {
    enabled: bool,
    stdin: Option<BufWriter<ChildStdin>>,
    stdout: Option<BufReader<ChildStdout>>,
    filter: Option<Rc<Filter>>,
}

impl ChatGPT {
    pub fn new(config: &Config, filter: Rc<Filter>) -> Self {
        if (!config.chatgpt.enabled) {
            Self {
                enabled: config.chatgpt.enabled,
                stdin: None,
                stdout: None,
                filter: None,
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
                filter: Some(filter),
            }
        }
    }

    pub fn complete(&mut self, prompt: &str) -> Option<String> {
        if (self.enabled) {
            let prompt = self.filter.as_ref().unwrap().sanitize(prompt);
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
