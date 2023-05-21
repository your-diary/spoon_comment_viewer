use std::{
    error::Error,
    sync::mpsc::{self, Receiver},
    thread,
};

use super::websocket;

pub struct WebSocket {
    rx: Option<Receiver<String>>,
}

impl WebSocket {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { rx: None }
    }

    pub fn connect(&mut self, live_id: u64) -> Result<(), Box<dyn Error>> {
        let url = format!("wss://jp-hala.spooncast.net/{}", live_id);

        let (tx, rx) = mpsc::channel();

        let on_open_message = format!(
            r#"
                {{
                    "live_id":    "{}",
                    "appversion": "8.3.3",
                    "retry":      0,
                    "reconnect":  true,
                    "event":      "live_join",
                    "type":       "live_req",
                    "useragent":  "Web"
                }}
            "#,
            live_id
        );

        let mut ws = websocket::WebSocket::new(tx, &url, Some(&on_open_message))?;
        thread::spawn(move || {
            let _ = ws.read_loop();
        });

        self.rx = Some(rx);
        Ok(())
    }

    pub fn fetch(&self) -> Vec<String> {
        let mut ret = vec![];
        while let Ok(s) = self.rx.as_ref().unwrap().try_recv() {
            ret.push(s);
        }
        ret
    }
}
