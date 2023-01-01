use super::config::Config;

use std::env;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::{ChildStdin, Command, Stdio};

pub struct CoeFont {
    enabled: bool,
    stdin: Option<BufWriter<ChildStdin>>,
}

impl CoeFont {
    pub fn new(config: &Config) -> Self {
        if (!config.coefont.enabled) {
            Self {
                enabled: config.coefont.enabled,
                stdin: None,
            }
        } else {
            env::set_current_dir(Path::new(&config.coefont.binary_path).parent().unwrap()).unwrap();
            let child = Command::new(&config.coefont.binary_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::null()) //discards stdout
                .spawn()
                .unwrap();
            Self {
                enabled: config.coefont.enabled,
                stdin: Some(BufWriter::new(child.stdin.unwrap())),
            }
        }
    }

    pub fn say(&mut self, prompt: &str) {
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
