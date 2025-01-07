use serde_json::Value;

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
    pub fn from(raw: &str) -> JsonPathRoot {
        let chars = raw.chars().collect::<Vec<_>>();

        unimplemented!()
    }

    pub fn extract(&self, value: &Value) -> Result<JsonPathResult, String> {
        unimplemented!()
    }
}
