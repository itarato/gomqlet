use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub url: String,
    pub headers: Vec<[String; 2]>,
}
