use std::{path::Path, time::Duration};

use regex::Regex;

//tilde expansion + makes it absolute path
pub fn canonicalize_path(s: &str) -> String {
    Path::new(&s.replace('~', &std::env::var("HOME").unwrap()))
        .canonicalize()
        .unwrap()
        .as_path()
        .to_str()
        .unwrap()
        .to_string()
}

pub fn canonicalize_path_in_place(s: &mut String) {
    *s = canonicalize_path(s);
}

pub fn pretty_print_duration(d: Duration) -> String {
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

pub fn is_japanese(s: &str) -> bool {
    let re = Regex::new(r#"[ぁ-んァ-ヶｱ-ﾝﾞﾟ一-龠]"#).unwrap();
    re.is_match(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pretty_print_duration() {
        assert_eq!("3秒", pretty_print_duration(Duration::from_secs(3)));
        assert_eq!("60秒", pretty_print_duration(Duration::from_secs(60)));
        assert_eq!("1分01秒", pretty_print_duration(Duration::from_secs(61)));
        assert_eq!("60分00秒", pretty_print_duration(Duration::from_secs(3600)));
        assert_eq!(
            "1時間00分01秒",
            pretty_print_duration(Duration::from_secs(3601))
        );
        assert_eq!(
            "1時間10分15秒",
            pretty_print_duration(Duration::from_secs(4215))
        );
    }

    #[test]
    fn test_is_japanese() {
        assert_eq!(false, is_japanese("hello"));
        assert_eq!(false, is_japanese("342352"));
        assert_eq!(false, is_japanese("🌙"));
        assert_eq!(false, is_japanese("사랑합니다"));
        assert_eq!(false, is_japanese("０１２３４"));
        assert_eq!(false, is_japanese("ｄｅｆｇｈ"));

        assert_eq!(true, is_japanese("你好"));
        assert_eq!(true, is_japanese("あ"));
        assert_eq!(true, is_japanese("ABCコンテスト"));
        assert_eq!(true, is_japanese("表現"));
    }
}
