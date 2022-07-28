use std::fmt;
use std::fmt::Display;

use chrono::offset::Local;

use crate::constant;

/*-------------------------------------*/

pub struct Comment {
    timestamp: String,
    user: String,
    text: String,
}

impl Comment {
    pub fn new(timestamp: String, user: String, text: String) -> Self {
        Comment {
            timestamp,
            user,
            text,
        }
    }
    pub fn user(&self) -> &str {
        &self.user
    }
}

impl Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}[{} ({})] {}: {}{}",
            constant::COLOR_PURPLE,
            Local::now().format("%H:%M:%S"),
            self.timestamp,
            self.user,
            constant::NO_COLOR,
            self.text
        )
    }
}

/*-------------------------------------*/

pub enum CommentType {
    Message,
    Combo,
    Unknown,
}

impl<'a> CommentType {
    const CLASS_NAME_MESSAGE: &'a str = " message";
    const CLASS_NAME_COMBO: &'a str = " combo";

    pub fn new(class_name: Option<String>) -> Self {
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

/*-------------------------------------*/
