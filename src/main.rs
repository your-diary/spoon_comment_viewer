use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ctrlc;
use thirtyfour_sync::ElementId;

use spoon_comment_viewer::config::Config;
use spoon_comment_viewer::selenium::Selenium;

const CONFIG_FILE: &str = "./config.json";

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();

    ctrlc::set_handler(move || {
        tx.send(0).unwrap();
    })
    .unwrap();

    let config = Config::new(CONFIG_FILE);

    let z = Selenium::new(
        config.webdriver_port(),
        Duration::from_millis(config.implicit_timeout_ms()),
    );

    spoon_comment_viewer::login(&z, config.twitter_id(), config.twitter_password())?;

    {
        print!("Press ENTER to continue: ");
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

    z.click("button[title='リスナー']")?; //opens the listeners tab in the sidebar

    //comments
    let mut comment_set: HashSet<ElementId> = HashSet::new(); //records existing comments
    let mut previous_author: String = String::new(); //for combo comment

    //listeners
    let mut previous_listeners_set: HashSet<String> = HashSet::new(); //for `いらっしゃい`, `おかえりなさい`, `またきてね`
    let mut previous_listeners_map: HashMap<String, Instant> = HashMap::new(); //for `xxx秒の滞在でした`
    let mut cumulative_listeners: HashSet<String> = HashSet::new(); //for `おかえりなさい`

    let mut c = -1;
    loop {
        c += 1;

        if (rx.try_recv().is_ok()) {
            break;
        }

        thread::sleep(Duration::from_millis(config.comment_check_interval_ms()));

        match spoon_comment_viewer::process_comment(
            &z,
            &config,
            &mut comment_set,
            &mut previous_author,
        ) {
            Err(e) => {
                println!("{}", e);
                continue;
            }
            _ => (),
        }

        //checks listeners every `comment_check_interval_ms * listener_check_interval_ratio` milliseconds
        if ((c as usize) % config.listener_check_interval_ratio() == 0) {
            match spoon_comment_viewer::process_listeners(
                &z,
                &config,
                /* is_first_call = */ c == 0,
                &mut previous_listeners_set,
                &mut previous_listeners_map,
                &mut cumulative_listeners,
            ) {
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
