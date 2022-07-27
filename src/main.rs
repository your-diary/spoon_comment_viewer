use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::thread;
use std::time::Duration;

use chrono::offset::Local;
use thirtyfour_sync::{error::WebDriverError, prelude::*, ElementId};

use spoon_comment_viewer::config::Config;

const NO_COLOR: &str = "\u{001B}[0m";
const COLOR: &str = "\u{001B}[095m";

struct Comment {
    timestamp: String,
    user: String,
    text: String,
}
impl Comment {
    fn new(timestamp: String, user: String, text: String) -> Self {
        Comment {
            timestamp,
            user,
            text,
        }
    }
}
impl Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}[{} ({})] {}: {}{}",
            COLOR,
            Local::now().format("%H:%M:%S"),
            self.timestamp,
            self.user,
            NO_COLOR,
            self.text
        )
    }
}

enum CommentType {
    Message,
    Combo,
    Unknown,
}
impl<'a> CommentType {
    const CLASS_NAME_MESSAGE: &'a str = " message";
    const CLASS_NAME_COMBO: &'a str = " combo";

    fn new(class_name: Option<String>) -> Self {
        match class_name {
            None => Self::Unknown,
            Some(s) => {
                if (s.ends_with(Self::CLASS_NAME_MESSAGE)) {
                    Self::Message
                } else if (s.ends_with(Self::CLASS_NAME_COMBO)) {
                    Self::Combo
                } else {
                    Self::Unknown
                }
            }
        }
    }
}

struct Selenium {
    driver: WebDriver,
}

impl Selenium {
    fn new(webdriver_port: usize, implicit_timeout: Duration) -> Self {
        let mut firefox = DesiredCapabilities::firefox();

        //disables desktop notification
        firefox
            .add_firefox_option(
                "prefs",
                serde_json::json!({"permissions.default.desktop-notification": 1}),
            )
            .unwrap();

        let driver = WebDriver::new(
            format!("http://localhost:{}", webdriver_port).as_str(),
            &firefox,
        )
        .unwrap();
        driver.set_implicit_wait_timeout(implicit_timeout).unwrap();

        Selenium { driver }
    }

    fn login(&self, twitter_id: &str, twitter_password: &str) -> Result<(), WebDriverError> {
        self.driver.get("https://www.spooncast.net/jp/")?;

        self.click(".btn-login")?;
        self.click(".btn-twitter button")?;

        self.switch_tab(1)?;

        self.input("#username_or_email", twitter_id)?;
        self.input("#password", twitter_password)?;
        self.click("#allow")?;

        self.switch_tab(0)?;

        Ok(())
    }

    fn query(&self, css_selector: &str) -> Result<WebElement, WebDriverError> {
        self.driver.find_element(By::Css(css_selector))
    }

    fn query_all(&self, css_selector: &str) -> Result<Vec<WebElement>, WebDriverError> {
        self.driver.find_elements(By::Css(css_selector))
    }

    fn click(&self, css_selector: &str) -> Result<(), WebDriverError> {
        self.query(css_selector).and_then(|e| e.click())
    }

    fn input(&self, css_selector: &str, s: &str) -> Result<(), WebDriverError> {
        self.query(css_selector).and_then(|e| e.send_keys(s))
    }

    fn inner_text(&self, css_selector: &str) -> Result<String, WebDriverError> {
        self.query(css_selector).and_then(|e| e.text())
    }

    fn switch_tab(&self, index: usize) -> Result<(), WebDriverError> {
        self.driver
            .switch_to()
            .window(&(self.driver.window_handles().unwrap()[index]))
    }
}

impl Drop for Selenium {
    fn drop(&mut self) {
        println!("Closing the driver...");
    }
}

const CONFIG_FILE: &str = "./config.json";

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new(CONFIG_FILE);

    let z = Selenium::new(
        config.webdriver_port(),
        Duration::from_millis(config.implicit_timeout_ms()),
    );

    z.login(config.twitter_id(), config.twitter_password())?;

    thread::sleep(Duration::from_secs(10));

    let mut comment_set: HashSet<ElementId> = HashSet::new();
    let mut previous_user: String = String::new(); //for combo comment

    loop {
        thread::sleep(Duration::from_millis(1000));

        let l = match z.query_all("li.chat-list-item.message, li.chat-list-item.combo") {
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
                    previous_user = comment.user.clone();
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
