use std::{error::Error, sync::Arc};

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::config::Config;

/*-------------------------------------*/

#[derive(Debug, Deserialize, Serialize)]
struct Req {
    model: String,
    prompt: String,
    temperature: f64,
    max_tokens: usize,
}
impl Req {
    fn new(model: &str, prompt: &str, temperature: f64, max_tokens: usize) -> Self {
        Self {
            model: model.to_string(),
            prompt: prompt.to_string(),
            temperature,
            max_tokens,
        }
    }
}

/*-------------------------------------*/

#[derive(Debug, Deserialize, Serialize)]
struct Res {
    choices: Vec<Choice>,
}
#[derive(Debug, Deserialize, Serialize)]
struct Choice {
    text: String,
}

/*-------------------------------------*/

pub async fn call(
    prompt: &str,
    config: Arc<Config>,
    client: Arc<Client>,
) -> Result<String, Box<dyn Error>> {
    let config = &config.chatgpt;

    let max_tokens = if (prompt.is_ascii()) {
        config.model.max_tokens_en
    } else {
        config.model.max_tokens_ja
    };

    let req = Req::new(
        &config.model.model,
        prompt,
        config.model.temperature,
        max_tokens,
    );

    let res: Response = client
        .post(&config.http.url)
        .body(serde_json::to_string(&req)?)
        .send()
        .await?;

    if res.status().is_success() {
        let text: String = res.text().await?;
        let res: Res = serde_json::from_str(&text)?;
        Ok(res
            .choices
            .into_iter()
            .next()
            .unwrap()
            .text
            .trim()
            .to_string())
    } else {
        Err(res.text().await?.into())
    }
}
