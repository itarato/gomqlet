use serde_json::Value;

use crate::util::Error;

pub enum JsonNest {
    List(usize),
    Key(String),
}

pub struct JsonPathRoot {
    nest: JsonNest,
}

pub enum JsonPathResult {
    String(String),
    Integer(i32),
}

impl JsonPathRoot {
    pub fn from(raw: &str) -> Result<JsonPathRoot, Error> {
        let chars = raw.chars().collect::<Vec<_>>();

        if chars[0] != '$' {
            return Err("".into());
        }

        Ok(JsonPathRoot {
            nest: JsonPathRoot::parse_value(&chars[1..])?,
        })
    }

    pub fn parse_value(chars: &[char]) -> Result<JsonNest, Error> {
        unimplemented!()
    }

    pub fn extract(&self, value: &Value) -> Result<JsonPathResult, Error> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_empty() {}
}
