use std::error::Error;
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use ctrlc;

use spoon_comment_viewer::config::Config;
use spoon_comment_viewer::spoon::Spoon;

const CONFIG_FILE: &str = "./config.json";

fn main() -> Result<(), Box<dyn Error>> {
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

    {
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

    spoon.init();

    let mut c = -1isize;
    loop {
        c += 1;

        if (rx.try_recv().is_ok()) {
            break;
        }

        thread::sleep(Duration::from_millis(
            config.spoon.comment_check_interval_ms,
        ));

        match spoon.process_comment(&config) {
            Err(e) => {
                println!("{}", e);
                continue;
            }
            _ => (),
        }

        //checks listeners every `comment_check_interval_ms * listener_check_interval_ratio` milliseconds
        if ((c as usize) % config.spoon.listener_check_interval_ratio == 0) {
            match spoon.process_listeners(&config) {
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
                _ => (),
            }
        }
    }

    Ok(())
}
