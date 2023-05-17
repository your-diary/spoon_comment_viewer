use itertools::Itertools;

pub fn prettier(res: String, max_tokens_en: usize, max_tokens_ja: usize) -> String {
    if (res.is_ascii()) {
        let mut res = res
            .split_whitespace()
            .take(max_tokens_en)
            .join(" ")
            .chars()
            .collect_vec();
        if let Some(i) = res.iter().rposition(|&c| c == '.' || c == '!' || c == '?') {
            res = res.into_iter().take(i + 1).collect_vec();
        }
        res.into_iter().join("")
    } else {
        let mut res = res
            .split_whitespace()
            .join(" ")
            .chars()
            .take(max_tokens_ja)
            .collect_vec();
        if let Some(i) = res
            .iter()
            .rposition(|&c| c == '。' || c == '！' || c == '？' || c == '!' || c == '?')
        {
            res = res.into_iter().take(i + 1).collect_vec();
        }
        res.into_iter().join("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // #[ignore]
    fn test01() {
        assert_eq!(
            "I'm five years old.",
            prettier("I'm five years old. Do you know me?".to_string(), 5, 0)
        );
    }

    #[test]
    // #[ignore]
    fn test02() {
        assert_eq!(
            "Hi, I'm five years old.",
            prettier("Hi, I'm five years old. Do you know me?".to_string(), 5, 0)
        );
    }

    #[test]
    // #[ignore]
    fn test03() {
        assert_eq!(
            "Hi, I'm five years",
            prettier("Hi, I'm five years old. Do you know me?".to_string(), 4, 0)
        );
    }

    #[test]
    // #[ignore]
    fn test04() {
        assert_eq!(
            "こんにちは、ワトソンくん。",
            prettier(
                "こんにちは、ワトソンくん。これは何ですか？".to_string(),
                0,
                13
            )
        );
    }

    #[test]
    // #[ignore]
    fn test05() {
        assert_eq!(
            "こんにちは、ワトソンくん。",
            prettier(
                "こんにちは、ワトソンくん。これは何ですか？".to_string(),
                0,
                14
            )
        );
    }

    #[test]
    // #[ignore]
    fn test06() {
        assert_eq!(
            "こんにちは",
            prettier(
                "こんにちは、ワトソンくん。これは何ですか？".to_string(),
                0,
                5
            )
        );
    }
}
