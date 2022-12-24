use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ctrlc;
use thirtyfour_sync::ElementId;

use spoon_comment_viewer::chatgpt::ChatGPT;
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

    let mut chatgpt = ChatGPT::new(&config);

    let z = Selenium::new(
        config.selenium.webdriver_port,
        Duration::from_millis(config.selenium.implicit_timeout_ms),
    );

    spoon_comment_viewer::login(&z, &config.twitter.id, &config.twitter.password)?;

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

    //tries to open the listeners tab in the sidebar
    //We intentionally ignore the result as this operation fails when the tab is already open.
    let _ = z.click("button[title='リスナー']");

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

        thread::sleep(Duration::from_millis(
            config.spoon.comment_check_interval_ms,
        ));

        let timestamp = match z.inner_text(".time-chip-container span") {
            Err(e) => {
                println!("{}", e);
                continue;
            }
            Ok(t) => t,
        };

        match spoon_comment_viewer::process_comment(
            &z,
            &config,
            &timestamp,
            &mut comment_set,
            &mut previous_author,
            &mut chatgpt,
        ) {
            Err(e) => {
                println!("{}", e);
                continue;
            }
            _ => (),
        }

        //checks listeners every `comment_check_interval_ms * listener_check_interval_ratio` milliseconds
        if ((c as usize) % config.spoon.listener_check_interval_ratio == 0) {
            match spoon_comment_viewer::process_listeners(
                &z,
                &config,
                /* is_first_call = */ c == 0,
                &timestamp,
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
