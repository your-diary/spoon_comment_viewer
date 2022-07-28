pub mod comment;
pub mod config;
pub mod selenium;

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use thirtyfour_sync::{error::WebDriverError, ElementId, WebDriverCommands};

use comment::{Comment, CommentType};
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

pub fn process_listeners(
    z: &Selenium,
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
            println!(
                "{}さん、また来てね。(滞在時間: {}秒)",
                e,
                previous_listeners_map.get(&e).unwrap().elapsed().as_secs()
            ); //TODO: convert to comment
               //TODO: pretty-print instead of `as_secs()`
            previous_listeners_map.remove(&e);
        } else {
            //unexpected to happen
            println!("{}さん、また来てね。", e); //TODO: convert to comment
        }
    }

    for e in new_listeners {
        previous_listeners_map.insert(e.clone(), Instant::now());
        if (cumulative_listeners.contains(&e)) {
            println!("{}さん、おかえりなさい。", e); //TODO: convert to comment
        } else {
            cumulative_listeners.insert(e.clone());
            println!("{}さん、いらっしゃい。", e); //TODO: convert to comment
        }
    }

    *previous_listeners_set = listeners_set;

    Ok(())
}

/*-------------------------------------*/

pub fn process_comment(
    z: &Selenium,
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
                println!("{}", comment);
                *previous_author = String::from(comment.user());
            }

            CommentType::Combo => {
                let comment = Comment::new(timestamp.clone(), previous_author.clone(), inner_text);
                println!("{}", comment);
            }

            CommentType::Unknown => continue,
        }
    }

    Ok(())
}

/*-------------------------------------*/
