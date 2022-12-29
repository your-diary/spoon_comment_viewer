use std::time::Duration;

pub fn tilde_expansion(s: &str) -> String {
    s.replace('~', &std::env::var("HOME").unwrap())
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
