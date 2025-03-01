use rand::prelude::*;
use std::{fmt::Display, fs::File, io::Read};

#[derive(Debug, Clone)]
pub struct CoordUsize {
    pub x: usize,
    pub y: usize,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn trim_coloured_string_list(elems: Vec<(String, Option<usize>)>, max_len: usize) -> String {
    let mut len = 0usize;
    let mut out_elems: Vec<String> = vec![];

    for (s, maybe_color) in elems {
        let used_len = (max_len - len).min(s.len());
        let formatted_s = if let Some(color) = maybe_color {
            format!("\x1B[{}m{}\x1B[0m", color, &s[0..used_len])
        } else {
            s[0..used_len].to_string()
        };
        out_elems.push(formatted_s);

        len += used_len;
        if len >= max_len {
            break;
        }
    }

    out_elems.join("")
}

pub fn fuzzy_match(subject: &str, pattern: &str) -> Option<Vec<usize>> {
    let subject = subject.to_lowercase();
    let pattern = pattern.to_lowercase();

    let pattern_chars = pattern.chars().collect::<Vec<_>>();
    if pattern.is_empty() {
        return Some(vec![]);
    }

    let mut match_points = vec![];
    let mut pattern_i = 0usize;

    for (i, subject_ch) in subject.chars().enumerate() {
        if subject_ch == pattern_chars[pattern_i] {
            match_points.push(i);
            pattern_i += 1;
        }

        if pattern_i == pattern.len() {
            return Some(match_points);
        }
    }

    None
}

pub fn random_integer(min: i32, max: i32) -> i32 {
    (random::<i32>() % (max - min)) + min
}

pub fn random_string(len: usize) -> String {
    (0..len)
        .into_iter()
        .map(|_| (random::<u8>() % (b'z' - b'a' + 1) + b'a') as char)
        .collect()
}

pub fn random_word() -> String {
    let mut content = String::new();
    File::open("./data/words.txt")
        .unwrap()
        .read_to_string(&mut content)
        .expect("Failed reading words file");

    let words = content.lines().map(|s| s.to_string()).collect::<Vec<_>>();

    let len = words.len();
    words[random::<usize>() % len].clone()
}

pub fn err_ctx<E: Display>(context: &str) -> impl FnOnce(E) -> Error + use<'_, E> {
    move |err| format!("{} (caused by: {})", context, err).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match_exact_match() {
        assert_eq!(fuzzy_match("hello", "hello"), Some(vec![0, 1, 2, 3, 4]));
    }

    #[test]
    fn test_fuzzy_match_subset() {
        assert_eq!(fuzzy_match("hello", "hlo"), Some(vec![0, 2, 4]));
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        assert_eq!(fuzzy_match("Hello", "hello"), Some(vec![0, 1, 2, 3, 4]));
        assert_eq!(fuzzy_match("WORLD", "world"), Some(vec![0, 1, 2, 3, 4]));
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        assert_eq!(fuzzy_match("hello", "world"), None);
    }

    #[test]
    fn test_fuzzy_match_empty_pattern() {
        assert_eq!(fuzzy_match("hello", ""), Some(vec![]));
    }

    #[test]
    fn test_fuzzy_match_pattern_longer_than_subject() {
        assert_eq!(fuzzy_match("hi", "hello"), None);
    }

    #[test]
    fn test_fuzzy_match_scattered_characters() {
        assert_eq!(fuzzy_match("fibonacci", "fbi"), Some(vec![0, 2, 8]));
    }

    #[test]
    fn test_trim_empty_list() {
        let input: Vec<(String, Option<usize>)> = vec![];
        assert_eq!(trim_coloured_string_list(input, 10), "");
    }

    #[test]
    fn test_single_uncolored_string() {
        let input = vec![("hello".to_string(), None)];
        assert_eq!(trim_coloured_string_list(input, 10), "hello");
    }

    #[test]
    fn test_single_colored_string() {
        let input = vec![("hello".to_string(), Some(31))]; // Red color
        assert_eq!(trim_coloured_string_list(input, 10), "\x1B[31mhello\x1B[0m");
    }

    #[test]
    fn test_multiple_strings_no_trimming() {
        let input = vec![
            ("hello".to_string(), None),
            ("world".to_string(), Some(32)), // Green color
        ];
        assert_eq!(
            trim_coloured_string_list(input, 10),
            "hello\x1B[32mworld\x1B[0m"
        );
    }

    #[test]
    fn test_trimming_single_string() {
        let input = vec![("hello world".to_string(), None)];
        assert_eq!(trim_coloured_string_list(input, 5), "hello");
    }

    #[test]
    fn test_trimming_multiple_strings() {
        let input = vec![
            ("hello".to_string(), None),
            ("world".to_string(), Some(32)), // Green color
        ];
        assert_eq!(
            trim_coloured_string_list(input, 7),
            "hello\x1B[32mwo\x1B[0m"
        );
    }

    #[test]
    fn test_zero_max_length() {
        let input = vec![("hello".to_string(), None)];
        assert_eq!(trim_coloured_string_list(input, 0), "");
    }

    #[test]
    fn test_multiple_colored_strings() {
        let input = vec![
            ("hello".to_string(), Some(31)), // Red color
            ("world".to_string(), Some(32)), // Green color
        ];
        assert_eq!(
            trim_coloured_string_list(input, 10),
            "\x1B[31mhello\x1B[0m\x1B[32mworld\x1B[0m"
        );
    }

    #[test]
    fn test_mixed_colored_and_uncolored() {
        let input = vec![
            ("hello".to_string(), Some(31)), // Red color
            ("world".to_string(), None),
            ("!".to_string(), Some(34)), // Blue color
        ];
        assert_eq!(
            trim_coloured_string_list(input, 11),
            "\x1B[31mhello\x1B[0mworld\x1B[34m!\x1B[0m"
        );
    }

    #[test]
    fn test_trim_within_colored_string() {
        let input = vec![("hello world".to_string(), Some(31))]; // Red color
        assert_eq!(
            trim_coloured_string_list(input, 5),
            "\x1B[31mhello\x1B[0m"
        );
    }
}