pub mod comment;
pub mod config;
pub mod constant;
pub mod selenium;

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use chrono::offset::Local;
use thirtyfour_sync::{error::WebDriverError, ElementId, WebDriverCommands};

use comment::{Comment, CommentType};
use config::Config;
use selenium::Selenium;

/*-------------------------------------*/

pub fn login(z: &Selenium, twitter_id: &str, twitter_password: &str) -> Result<(), WebDriverError> {
    z.driver().get("https://www.spooncast.net/jp/")?;

    z.click(".btn-login")?;
    z.click(".btn-twitter button")?;

    z.switch_tab(1)?;

    z.input("#username_or_email", twitter_id)?;
    z.input("#password", twitter_password)?;
    z.click("#allow")?;

    z.switch_tab(0)?;

    Ok(())
}

/*-------------------------------------*/

pub fn comment(z: &Selenium, s: &str) -> Result<(), WebDriverError> {
    z.input("textarea", s)?;
    z.click("button[title='送信']")?;
    Ok(())
}

/*-------------------------------------*/

fn print_comment(c: &Comment) {
    println!(
        "{}[{} ({})]{}{} {}: {}{}",
        constant::COLOR_BLACK,
        Local::now().format("%H:%M:%S"),
        c.timestamp(),
        constant::NO_COLOR,
        constant::COLOR_PURPLE,
        c.user(),
        constant::NO_COLOR,
        c.text(),
    );
}

fn print_listener(s: &str) {
    println!(
        "{}[{}]{}{} {}{}",
        constant::COLOR_BLACK,
        Local::now().format("%H:%M:%S"),
        constant::NO_COLOR,
        constant::COLOR_GREEN,
        s,
        constant::NO_COLOR,
    );
}

/*-------------------------------------*/

pub fn process_comment(
    z: &Selenium,
    config: &Config,
    comment_set: &mut HashSet<ElementId>, //records existing comments
    previous_author: &mut String,         //for combo comment
) -> Result<(), WebDriverError> {
    let l = z.query_all("li.chat-list-item")?;

    let timestamp = z.inner_text(".time-chip-container span")?;

    let num_new_comment = {
        let mut c = 0;
        for e in l.iter().rev() {
            if (comment_set.contains(&e.element_id)) {
                break;
            }
            comment_set.insert(e.element_id.clone());
            c += 1;
        }
        c
    };

    for e in l.iter().skip(l.len() - num_new_comment) {
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
                print_comment(&comment);
                *previous_author = String::from(comment.user());
            }

            CommentType::Combo => {
                let comment = Comment::new(timestamp.clone(), previous_author.clone(), inner_text);
                print_comment(&comment);
            }

            CommentType::Unknown => continue,
        }
    }

    Ok(())
}

/*-------------------------------------*/

fn pretty_print_duration(d: Duration) -> String {
    let s = d.as_secs();
    if (s <= 60) {
        format!("{}秒", s)
    } else if (s <= 3600) {
        let min = s / 60;
        let sec = s - min * 60;
        format!("{}分{:02}秒", min, sec)
    } else {
        let hour = s / 3600;
        let min = (s - hour * 3600) / 60;
        let sec = s - hour * 3600 - min * 60;
        format!("{}時間{:02}分{:02}秒", hour, min, sec)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pretty_print_duration() {
        assert_eq!(
            "3秒",
            super::pretty_print_duration(super::Duration::from_secs(3))
        );
        assert_eq!(
            "60秒",
            super::pretty_print_duration(super::Duration::from_secs(60))
        );
        assert_eq!(
            "1分01秒",
            super::pretty_print_duration(super::Duration::from_secs(61))
        );
        assert_eq!(
            "60分00秒",
            super::pretty_print_duration(super::Duration::from_secs(3600))
        );
        assert_eq!(
            "1時間00分01秒",
            super::pretty_print_duration(super::Duration::from_secs(3601))
        );
        assert_eq!(
            "1時間10分15秒",
            super::pretty_print_duration(super::Duration::from_secs(4215))
        );
    }
}

pub fn process_listeners(
    z: &Selenium,
    config: &Config,
    is_first_call: bool,
    previous_listeners_set: &mut HashSet<String>, //for `いらっしゃい`, `おかえりなさい`, `またきてね`
    previous_listeners_map: &mut HashMap<String, Instant>, //for `xxx秒の滞在でした`
    cumulative_listeners: &mut HashSet<String>,   //for `おかえりなさい`
) -> Result<(), WebDriverError> {
    z.click("div.user-list-wrap button[title='再読み込み']")?;

    //retrieves the list of the names of current listeners
    //
    //We can instead GET `https://jp-api.spooncast.net/lives/<live_id>/listeners/` to retrieve
    // the list of listeners where `<live_id>` can be extracted from `SPOONCAST_JP_liveCurrentInfo`
    // in local storage.
    //It is of the form `{"30538814":{"uId":"l63m46d6","created":"2022-07-27T11:30:12.193915Z"}}`.
    let listeners_set: HashSet<String> = {
        let mut listeners_list = Vec::new();
        let l = z.query_all("button p.name.text-box")?;
        for e in l {
            match e.text() {
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
                Ok(s) => listeners_list.push(s),
            }
        }
        HashSet::from_iter(listeners_list.into_iter())
    };

    let exited_listeners = &*previous_listeners_set - &listeners_set;
    let new_listeners = &listeners_set - &previous_listeners_set;

    for e in exited_listeners {
        if (previous_listeners_map.contains_key(&e)) {
            let c = format!(
                "{}さん、また来てね。(滞在時間: {})",
                e,
                pretty_print_duration(previous_listeners_map.get(&e).unwrap().elapsed()),
            );
            print_listener(&c);
            if (config.should_comment_listener()) {
                comment(&z, &c)?;
            }
            previous_listeners_map.remove(&e);
        } else {
            //unexpected to happen
            let c = format!("{}さん、また来てね。", e);
            print_listener(&c);
            if (config.should_comment_listener()) {
                comment(&z, &c)?;
            }
        }
    }

    for e in new_listeners {
        previous_listeners_map.insert(e.clone(), Instant::now());
        if (cumulative_listeners.contains(&e)) {
            let c = format!("{}さん、おかえりなさい。", e);
            print_listener(&c);
            if (config.should_comment_listener()) {
                comment(&z, &c)?;
            }
        } else {
            cumulative_listeners.insert(e.clone());
            if (!is_first_call) {
                let c = format!("{}さん、いらっしゃい。", e);
                print_listener(&c);
                if (config.should_comment_listener()) {
                    comment(&z, &c)?;
                }
            }
        }
    }

    *previous_listeners_set = listeners_set;

    Ok(())
}

/*-------------------------------------*/
