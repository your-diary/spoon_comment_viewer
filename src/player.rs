use std::process::{Child, Command, Stdio};

pub struct Audio {
    path: String,
    volume: f64,
    should_reverb: bool,
    should_repeat: bool,
}

impl Audio {
    pub fn new(path: &str, volume: f64, should_reverb: bool, should_repeat: bool) -> Self {
        Self {
            path: path.to_string(),
            volume,
            should_reverb,
            should_repeat,
        }
    }
}

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
    pub fn play(&mut self, audio: &Audio) {
        let mut args = vec![format!("-v {}", audio.volume), audio.path.clone()];
        if (audio.should_reverb) {
            args.push("pad".to_string());
            args.push("0".to_string());
            args.push("2".to_string());
            args.push("reverb".to_string());
        }
        if (audio.should_repeat) {
            args.push("repeat".to_string());
            args.push("-".to_string());
        }
        if let Ok(c) = Command::new("play")
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            self.children.push(c);
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
