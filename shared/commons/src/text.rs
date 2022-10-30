pub fn shorten(text: &str, limit: u16) -> &str {
    let words = words(text);
    let mut len = 0;
    let mut i = 0;

    while i < words.len() && (len + words[i].len() + 1) < limit as usize {
        len += words[i].len() + (if i == 0 { 0 } else { 1 });
        i += 1;
    }
    &text[0..len]
}

fn words(input: &str) -> Vec<&str> {
    input.split(' ').collect()
}

#[cfg(test)]
mod test {
    use super::shorten;

    #[test]
    fn test_short_returns_same_if_short() {
        let short_text = "This is some text.";
        assert_eq!(shorten(short_text, 100), short_text)
    }

    #[test]
    fn test_shorten_returns_shortened_sentence_limit_on_dot() {
        let text = "This is some text. Looooong word.";
        assert_eq!(shorten(text, 18), "This is some")
    }

    #[test]
    fn test_shorten_returns_shortened_sentence_limit_after_dot() {
        let text = "This is some text. Looooong word.";
        assert_eq!(shorten(text, 19), "This is some text.")
    }

    #[test]
    fn test_shorten_returns_shortened_sentence_limit_with_ellipsis() {
        let text = "This is some text. Looooong word.";
        assert_eq!(shorten(text, 21), "This is some text.")
    }

    #[test]
    fn test_shorten_returns_shortened_sentence_limit_with_ellipsis_longer() {
        let text = "This is some text. Looooong word.";
        assert_eq!(shorten(text, 23), "This is some text.")
    }
}
