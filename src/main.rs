use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use log::error;

use spoon_comment_viewer::config::Config;
use spoon_comment_viewer::spoon_client::SpoonClient;

const CONFIG_FILE: &str = "./config.json";

fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let (tx, rx) = mpsc::channel();

    ctrlc::set_handler(move || {
        tx.send(0).unwrap();
    })
    .unwrap();

    let config = Rc::new(Config::new(CONFIG_FILE));

    let mut spoon = SpoonClient::new(config.clone());
    spoon.login(
        &config.spoon.url,
        &config.twitter.id,
        &config.twitter.password,
    )?;

    thread::sleep(Duration::from_millis(3000));

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

    thread::sleep(Duration::from_millis(5000));

    spoon.init()?;

    let start = Instant::now();
    let mut timer = Instant::now();
    loop {
        thread::sleep(Duration::from_millis(
            config.spoon.comment_check_interval_ms,
        ));

        if ((start.elapsed() > Duration::from_secs(3600 * 2 + 5)) || rx.try_recv().is_ok()) {
            break;
        }

        if (timer.elapsed() > Duration::from_millis(config.spoon.listener_check_interval_ms)) {
            timer = Instant::now();

            println!("listener"); //TODO

            if let Err(e) = spoon.process_listeners(&config) {
                error!("{}", e);
                continue;
            }

            if let Err(e) = spoon.process_message_tunnel() {
                error!("{}", e);
                continue;
            }
        }

        let ins = Instant::now();
        let s = spoon.process_comments();
        println!("{:?}", ins.elapsed());
        if let Err(s) = s {
            error!("{}", s);
        }

        //TODO
        //         if let Err(e) = spoon.process_comments() {
        //             error!("{}", e);
        //             continue;
        //         }
    }

    Ok(())
}
