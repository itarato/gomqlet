#[derive(Debug, Clone)]
pub struct CoordUsize {
    pub x: usize,
    pub y: usize,
}

pub fn trim_string(s: &str, len: usize) -> &str {
    &s[0..s.len().min(len)]
}
