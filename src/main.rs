use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

use thirtyfour_sync::ElementId;

use spoon_comment_viewer::config::Config;
use spoon_comment_viewer::selenium::Selenium;

const CONFIG_FILE: &str = "./config.json";

fn main() -> Result<(), Box<dyn Error>> {
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
}

//     println!();
//     driver.set_implicit_wait_timeout(Duration::from_millis(10));
//     loop {
//         print!("\n> ");
//         io::stdout().flush().unwrap();
//         let mut buf = String::new();
//         io::stdin().read_line(&mut buf).unwrap();
//         buf = buf.trim().to_string();
//         if (buf == "q") {
//             break;
//         }
//         let tokens: Vec<&str> = buf.split_whitespace().collect();
//         if (tokens.len() == 2 && tokens[0] == "click") {
//             match driver.find_element(By::Css(tokens[1])) {
//                 Ok(e) => match e.click() {
//                     Err(e) => println!("{}", e),
//                     _ => (),
//                 },
//                 Err(e) => println!("{}", e),
//             }
//         } else if (tokens.len() == 3 && tokens[0] == "input") {
//             match driver.find_element(By::Css(tokens[1])) {
//                 Ok(e) => match e.send_keys(tokens[2]) {
//                     Err(e) => println!("{}", e),
//                     _ => (),
//                 },
//                 Err(e) => println!("{}", e),
//             }
//         }
//     }

//     #allow
//     sleep();
//     e.send_keys(TypingData::from(Keys::Control) + 'a').unwrap(); //Ctrl+a
//     sleep();

//     //JavaScriptを実行し、返り値を受け取る
//     let json = driver
//         .execute_script(r"return {a: 'hello', b: [1, 2, 3]};")
//         .unwrap()
//         .value()
//         .clone();
//     //展開方法1
//     json.get("b")
//         .unwrap()
//         .as_array()
//         .unwrap()
//         .iter()
//         .for_each(|e| {
//             println!("{}", e.as_f64().unwrap()); //=> 1 2 3
//         });
//     //展開方法2
//     if let Value::Object(o) = json {
//         o.iter().for_each(|(key, value)| {
//             println!("key: {}", key);
//             match value {
//                 Value::String(s) => println!("value: {}", s),
//                 Value::Array(v) => {
//                     v.iter().for_each(|e| {
//                         if let Value::Number(n) = e {
//                             println!("value: {}", n.as_f64().unwrap());
//                         }
//                     });
//                 }
//                 _ => panic!(),
//             }
//         });
//     }
//     //=> key: a
//     //   value: hello
//     //   key: b
//     //   value: 1
//     //   value: 2
//     //   value: 3
//
//     //新規タブを開き、移動する
//     driver
//         .execute_script(r"window.open('https://google.com');")
//         .unwrap();
//     driver
//         .switch_to()
//         .window(&(driver.window_handles().unwrap()[1]))
//         .unwrap();
//     sleep();
//
//     //リロード、進む、戻る
//     driver.get("https://yahoo.co.jp/").unwrap();
//     driver.refresh().unwrap();
//     driver.back().unwrap();
//     driver.forward().unwrap();
//     sleep();
//     driver
//         .switch_to()
//         .window(&(driver.window_handles().unwrap()[0]))
//         .unwrap();
//
//     //iframeへの移動
//     driver
//         .switch_to()
//         .frame_element(&(driver.find_element(By::Tag("iframe")).unwrap()))
//         .unwrap();
//     let e = driver.find_element(By::Tag("div")).unwrap();
//     println!("{}", e.text().unwrap()); //=> こんにちは
//     driver.switch_to().parent_frame().unwrap(); //親フレームへ戻る
//     sleep();
//
//     //要素の検索1 (inner_textやinnerHTMLの取得)
//     let e = driver.find_element(By::Css("div")).unwrap();
//     println!("{}", e.text().unwrap()); //=> "hello world"
//     println!("{}", e.inner_html().unwrap()); //=> "<div>hello</div>world"
//     println!("{}", e.outer_html().unwrap()); //=> "<div><div>hello</div>world</div>"
//     let inner = e.find_element(By::Css("div")).unwrap();
//     println!("{}", inner.text().unwrap()); //=> "hello"
//     sleep();
//
//     //要素の検索2 (属性やCSSの取得)
//     let e = driver.find_element(By::Css("a")).unwrap();
//     println!("{}", e.tag_name().unwrap()); //=> a
//     println!("{}", e.class_name().unwrap().unwrap()); //=> class1 class2
//     println!("{}", e.id().unwrap().unwrap()); //=> id1
//     println!("{}", e.get_attribute("href").unwrap().unwrap()); //=> url1
//     println!("{}", e.get_css_property("color").unwrap()); //=> rgb(255, 0, 0)
//     sleep();
//
//     //要素の検索4 (キー入力)
//     let e = driver.find_element(By::Css("body")).unwrap();
//     e.send_keys("a").unwrap(); //a
//     sleep();
//     e.send_keys(TypingData::from(Keys::Control) + 'a').unwrap(); //Ctrl+a
//     sleep();

//     driver.quit().unwrap();
