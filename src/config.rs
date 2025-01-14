use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct Config {
    pub url: String,
    pub headers: Vec<[String; 2]>,
    pub schema_cache: String,
    pub variables: Option<Value>,
}
