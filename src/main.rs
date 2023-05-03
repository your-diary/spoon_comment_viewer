use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use log::error;

use spoon_comment_viewer::config::Config;
use spoon_comment_viewer::spoon::Spoon;

const CONFIG_FILE: &str = "./config.json";

fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let (tx, rx) = mpsc::channel();

    ctrlc::set_handler(move || {
        tx.send(0).unwrap();
    })
    .unwrap();

    let config = Config::new(CONFIG_FILE);

    let mut spoon = Spoon::new(&config);
    spoon.login(
        &config.spoon.url,
        &config.twitter.id,
        &config.twitter.password,
    )?;

    //automatically starts a live
    if (config.spoon.live.enabled) {
        spoon.start_live(&config)?;
    //manually starts a live
    } else {
        print!("Press ENTER after you have started a live: ");
        io::stdout().flush().unwrap();
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        if (buf.trim() == "q") {
            return Ok(());
        }
    }

    if (rx.try_recv().is_ok()) {
        return Ok(());
    }

    spoon.init()?;

    let start = Instant::now();
    let mut c = -1isize;
    loop {
        c += 1;

        if ((start.elapsed() > Duration::from_secs(3600 * 2 + 5)) || rx.try_recv().is_ok()) {
            break;
        }

        thread::sleep(Duration::from_millis(
            config.spoon.comment_check_interval_ms,
        ));

        if let Err(e) = spoon.process_comments(&config) {
            error!("{}", e);
            continue;
        }

        //checks listeners every `comment_check_interval_ms * listener_check_interval_ratio` milliseconds
        if ((c as usize) % config.spoon.listener_check_interval_ratio == 0) {
            if let Err(e) = spoon.process_listeners(&config) {
                error!("{}", e);
                continue;
            }
        }

        if let Err(e) = spoon.process_message_tunnel(&config) {
            error!("{}", e);
            continue;
        }
    }

    Ok(())
}
