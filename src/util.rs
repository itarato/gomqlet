#[derive(Debug, Clone)]
pub struct CoordUsize {
    pub x: usize,
    pub y: usize,
}

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
