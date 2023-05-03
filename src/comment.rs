use std::fmt::{self, Display};

use thirtyfour_sync::ElementId;

use super::constant;

/*-------------------------------------*/

pub struct Comment {
    element_id: ElementId,
    comment_type: CommentType,
    user: Option<String>,
    text: Option<String>,
}

impl Comment {
    pub fn new(
        element_id: ElementId,
        comment_type: CommentType,
        user: Option<String>,
        text: Option<String>,
    ) -> Self {
        Comment {
            element_id,
            comment_type,
            user,
            text,
        }
    }
    pub fn element_id(&self) -> &ElementId {
        &self.element_id
    }
    pub fn comment_type(&self) -> &CommentType {
        &self.comment_type
    }
    pub fn user(&self) -> Option<&String> {
        self.user.as_ref()
    }
    pub fn text(&self) -> Option<&String> {
        self.text.as_ref()
    }
}

impl Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}:{} {}",
            constant::COLOR_PURPLE,
            self.user().unwrap_or(&String::new()),
            constant::NO_COLOR,
            self.text().unwrap_or(&String::new()),
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
