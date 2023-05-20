use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use log::{error, info};
use reqwest::header::HeaderMap;
use reqwest::Client;

use crate::chatgpt::{call, util};

use super::super::config::Config;
use super::super::filter::Filter;
use super::super::voicevox::Script;

async fn caller(
    index: usize,
    mut script: Script,
    buf: Arc<Mutex<Vec<Option<Script>>>>,
    config: Arc<Config>,
    client: Arc<Client>,
) {
    let start = Instant::now();
    let res = (|| async {
        let mut has_retried = false;
        loop {
            let start = Instant::now();
            match call::call(&script.script, config.clone(), client.clone()).await {
                Ok(s) => return s,
                Err(e) => {
                    if (e.to_string().contains(r#""type": "insufficient_quota""#)) {
                        return "QUOTA_ERROR".to_string();
                    } else {
                        error!("{}", e);
                        if (!has_retried
                            && (start.elapsed()
                                < Duration::from_millis(config.chatgpt.http.timeout_ms / 3)))
                        {
                            info!("Retrying...");
                            has_retried = true;
                            continue;
                        } else {
                            return "ERROR".to_string();
                        }
                    }
                }
            }
        }
    })()
    .await;
    let elapsed = start.elapsed();

    script.script = res;

    let mut buf = buf.lock().unwrap();
    info!("ChatGPT: {}ms", elapsed.as_millis());
    buf[index] = Some(script);
}

async fn chatgpt_thread(
    rx: Receiver<(usize, Script)>,
    buf: Arc<Mutex<Vec<Option<Script>>>>,
    config: Arc<Config>,
) {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        format!("Bearer {}", config.chatgpt.api_key)
            .parse()
            .unwrap(),
    );
    let client = Arc::new(
        Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_millis(config.chatgpt.http.timeout_ms))
            .build()
            .unwrap(),
    );

    loop {
        let (index, script) = match rx.recv() {
            Err(_) => return,
            Ok(r) => r,
        };
        tokio::spawn(caller(
            index,
            script,
            buf.clone(),
            config.clone(),
            client.clone(),
        ));
    }
}

//This struct implements a ChatGPT client, which is fully asynchronous and preserves the insertion order.
//To realize both of the properties, we utilize the vector `buf` and the two indices: `next_index` and `next_unread_index`.
//Initially, all of the elements of `buf` take the value `None` and the indices take `0`.
//When `push()` is called, ChatGPT is called asynchronously and the result is written as `Some` to `buf[next_index]`.
//By incrementing `next_index` every time `push()` is called, we can record every result independently.
//When `fetch()` is called,
//(a) If `buf[next_unread_index] == None`, we do nothing (even if, for example, `buf[next_unread_index + 1] == Some`).
//(b) Otherwise, we gather `buf[next_unread_index]`, `buf[next_unread_index + 1]`, ...
//     while the values are `Some`, return the results and increment `next_unread_index` by the length of the results.
//
//For example, assume `push()` has been called 4 times.
//So the value of `next_index` is `4`.
//However, `next_unread_index` is still `0` because the 1st call hasn't been completed (though the 2nd and the 4th calls have completed).
//
//                                   next_index
//                                    ↓
//          [None, Some, None, Some, None, ...]
//            ↑
//        next_unread_index
//
//When the 1st call has completed, now the value of `buf[next_unread_index]` has become `Some`.
//
//                                   next_index
//                                    ↓
//          [Some, Some, None, Some, None, ...]
//            ↑
//        next_unread_index
//
//So we can gather the first two values and return them when `fetch()` is called and increment `next_unread_index` by two.
//
//                                   next_index
//                                    ↓
//          [Some, Some, None, Some, None, ...]
//                        ↑
//                       next_unread_index
//
pub struct ChatGPT {
    tx: Option<Sender<(usize, Script)>>,

    config: Arc<Config>,

    filter: Filter,

    next_index: usize,
    next_unread_index: usize,
    buf: Arc<Mutex<Vec<Option<Script>>>>,
}

impl ChatGPT {
    pub fn new(config: &Config, filter: Filter) -> Self {
        let config = Arc::new(config.clone());
        if (!config.chatgpt.enabled) {
            Self {
                tx: None,
                config,
                filter,
                next_index: 0,
                next_unread_index: 0,
                buf: Arc::new(Mutex::new(vec![])),
            }
        } else {
            let (tx, rx) = mpsc::channel();
            let buf = Arc::new(Mutex::new(vec![None; 50000]));
            {
                let buf = buf.clone();
                let config = config.clone();
                thread::spawn(move || {
                    let runtime = tokio::runtime::Runtime::new().unwrap();
                    runtime.block_on(async move {
                        chatgpt_thread(rx, buf, config.clone()).await;
                    });
                });
            }
            Self {
                tx: Some(tx),
                config,
                filter,
                next_index: 0,
                next_unread_index: 0,
                buf,
            }
        }
    }

    pub fn push(&mut self, mut script: Script) {
        if (self.tx.is_none()) {
            return;
        }
        if (!self.filter.is_normal(&script.script)) {
            let original = script.script.clone();
            let sanitized = self.filter.sanitize(&script.script);
            info!(
                "Forbidden word sanitized: [{}] -> [{}]",
                original, sanitized
            );
            script.script = sanitized;
        }
        self.tx
            .as_ref()
            .unwrap()
            .send((self.next_index, script))
            .unwrap();
        self.next_index += 1;
    }

    pub fn fetch(&mut self) -> Vec<Script> {
        if (self.tx.is_none()) {
            return vec![];
        }
        let mut ret = vec![];
        let buf = self.buf.lock().unwrap();
        while let Some(s) = &buf[self.next_unread_index] {
            self.next_unread_index += 1;
            let mut s = s.clone();
            s.script = util::prettier(
                s.script,
                self.config.chatgpt.model.max_tokens_en,
                self.config.chatgpt.model.max_tokens_ja,
            );
            ret.push(s.clone());
        }
        ret
    }
}
