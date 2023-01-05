pub struct Filter {
    forbidden_words: Vec<String>,
}

//filters out forbidden words from input string
impl Filter {
    pub fn new(forbidden_words: &[String]) -> Self {
        Self {
            forbidden_words: forbidden_words.to_owned(),
        }
    }

    pub fn is_normal(&self, s: &str) -> bool {
        !self.forbidden_words.iter().any(|w| s.contains(w))
    }

    pub fn sanitize(&self, s: &str) -> String {
        let mut ret = s.to_string();
        for w in &self.forbidden_words {
            if (ret.contains(w)) {
                println!("{}", w);
                println!("{}", ret);
                ret = ret.replace(w, &"*".repeat(w.chars().count()));
                println!("{}", ret);
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // #[ignore]
    fn test01() {
        let filter = Filter::new(&vec![]);
        assert_eq!(true, filter.is_normal("あいうえお"));
    }

    #[test]
    // #[ignore]
    fn test02() {
        let filter = Filter::new(&vec!["りんご".to_string(), "ゴリラ".to_string()]);
        assert_eq!(true, filter.is_normal("あいうえお"));
        assert_eq!(false, filter.is_normal("ありんごい"));
        assert_eq!(false, filter.is_normal("あゴリラい"));
        assert_eq!(false, filter.is_normal("あゴリラりんごい"));
    }

    #[test]
    // #[ignore]
    fn test03() {
        let filter = Filter::new(&vec!["りんご".to_string(), "ボックス".to_string()]);
        assert_eq!("あいうえお".to_string(), filter.sanitize("あいうえお"));
        assert_eq!("あ***い".to_string(), filter.sanitize("ありんごい"));
        assert_eq!("あ****い".to_string(), filter.sanitize("あボックスい"));
        assert_eq!("あ****う***い", filter.sanitize("あボックスうりんごい"));
    }
}
