use std::{error::Error, rc::Rc};

use chrono::Local;
use log::error;

use super::constant;
use super::selenium::Selenium;

pub struct Logger {
    z: Rc<Selenium>,

    timestamp: String,
    ranking: String,
    num_spoon: String,
    num_heart: String,
    num_current_listener: String,
    num_total_listener: String,
}

impl Logger {
    pub fn new(z: Rc<Selenium>) -> Self {
        Self {
            z,

            timestamp: String::new(),
            ranking: String::new(),
            num_spoon: String::new(),
            num_heart: String::new(),
            num_current_listener: String::new(),
            num_total_listener: String::new(),
        }
    }

    fn refresh(&mut self) -> Result<(), Box<dyn Error>> {
        self.timestamp = self
            .z
            .inner_text(".time-chip-container span")?
            .trim()
            .to_string();
        if (self.timestamp.len() == 5) {
            self.timestamp = format!("00:{}", self.timestamp);
        }

        let count_info_list = self.z.query_all("ul.count-info-list li")?;
        let mut count_info_list_str = vec![];
        for e in count_info_list {
            count_info_list_str.push(e.text()?.trim().to_string());
        }
        match count_info_list_str.len() {
            //followers-only stream (ranking is not shown)
            4 => {
                count_info_list_str.insert(0, "?".to_string());
            }
            //normal streaming
            5 => {
                //do nothing
            }
            _ => {
                error!(
                    "`count_info_list` is of an unexpected form. Its length is {}.",
                    count_info_list_str.len()
                );
                for _ in 0..(5 - count_info_list_str.len()) {
                    count_info_list_str.insert(0, "?".to_string());
                }
            }
        }
        (
            self.ranking,
            self.num_spoon,
            self.num_heart,
            self.num_current_listener,
            self.num_total_listener,
        ) = (
            count_info_list_str[0].clone(),
            count_info_list_str[1].clone(),
            count_info_list_str[2].clone(),
            count_info_list_str[3].clone(),
            count_info_list_str[4].clone(),
        );

        Ok(())
    }

    pub fn log(&mut self, color: Option<&str>, s: &str) -> Result<(), Box<dyn Error>> {
        self.refresh()?;

        println!(
            "{}[{} ({}) ({}/{}/{}/{}/{})]{}{} {}{}",
            constant::COLOR_BLACK,
            Local::now().format("%H:%M:%S"),
            self.timestamp,
            self.ranking,
            self.num_spoon,
            self.num_heart,
            self.num_current_listener,
            self.num_total_listener,
            constant::NO_COLOR,
            color.unwrap_or_default(),
            s.replace('\n', "\\n"), //makes it a single line
            if (color.is_some()) {
                constant::NO_COLOR
            } else {
                ""
            }
        );

        Ok(())
    }
}
