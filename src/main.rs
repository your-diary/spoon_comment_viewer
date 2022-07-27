use std::collections::HashSet;
use std::error::Error;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use thirtyfour_sync::ElementId;

use spoon_comment_viewer::comment::{Comment, CommentType};
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
    }

    let mut comment_set: HashSet<ElementId> = HashSet::new(); //records existing comments
    let mut previous_user: String = String::new(); //for combo comment

    loop {
        thread::sleep(Duration::from_millis(config.comment_check_interval_ms()));

        let l = match z.query_all("li.chat-list-item") {
            Err(e) => {
                println!("{}", e);
                continue;
            }
            Ok(l) => l,
        };

        let timestamp = match z.inner_text(".time-chip-container span") {
            Ok(s) => s,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        for e in l {
            if (comment_set.contains(&e.element_id)) {
                continue;
            }
            comment_set.insert(e.element_id.clone());

            let inner_text = match e.text() {
                Ok(s) => s,
                Err(_) => continue,
            };

            let class_name = match e.class_name() {
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
                Ok(s) => s,
            };

            match CommentType::new(class_name) {
                CommentType::Message => {
                    let tokens: Vec<&str> = inner_text.splitn(2, "\n").collect();
                    if (tokens.len() != 2) {
                        println!("Comment [ {} ] has an invalid form.", inner_text);
                        continue;
                    }
                    let comment = Comment::new(
                        timestamp.clone(),
                        tokens[0].to_string(),
                        tokens[1].to_string(),
                    );
                    println!("{}", comment);
                    previous_user = String::from(comment.user());
                }

                CommentType::Combo => {
                    let comment =
                        Comment::new(timestamp.clone(), previous_user.clone(), inner_text);
                    println!("{}", comment);
                }

                CommentType::Unknown => continue,
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
