use std::fmt::{self, Display};

use super::constant;

/*-------------------------------------*/

pub struct Comment {
    user: String,
    text: String,
}

impl Comment {
    pub fn new(user: String, text: String) -> Self {
        Comment { user, text }
    }
    pub fn user(&self) -> &str {
        &self.user
    }
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}:{} {}",
            constant::COLOR_PURPLE,
            self.user,
            constant::NO_COLOR,
            self.text,
        )
    }
}

/*-------------------------------------*/

#[derive(PartialEq, Eq)]
pub enum CommentType {
    Message, //normal comment
    Combo, //When a single user posts comments in a row, all of the comments except the first one are "combo".
    Like,  //heart
    Present, //spoon, buster
    Guide, //`配信終了10分前だよ！` etc.
    Block, //`○○さんが強制退室になりました。`
    Unknown,
}

impl<'a> CommentType {
    const CLASS_NAME_MESSAGE: &'a str = " message";
    const CLASS_NAME_COMBO: &'a str = " combo";
    const CLASS_NAME_LIKE: &'a str = " like";
    const CLASS_NAME_PRESENT: &'a str = " present";
    const CLASS_NAME_GUIDE: &'a str = " guide";
    const CLASS_NAME_BLOCK: &'a str = " block";

    pub fn new(class_name: Option<String>) -> Self {
        match class_name {
            None => Self::Unknown,
            Some(s) => {
                if (s.ends_with(Self::CLASS_NAME_MESSAGE)) {
                    Self::Message
                } else if (s.ends_with(Self::CLASS_NAME_COMBO)) {
                    Self::Combo
                } else if (s.ends_with(Self::CLASS_NAME_LIKE)) {
                    Self::Like
                } else if (s.ends_with(Self::CLASS_NAME_PRESENT)) {
                    Self::Present
                } else if (s.ends_with(Self::CLASS_NAME_GUIDE)) {
                    Self::Guide
                } else if (s.ends_with(Self::CLASS_NAME_BLOCK)) {
                    Self::Block
                } else {
                    Self::Unknown
                }
            }
        }
    }
}

/*-------------------------------------*/
