use serde_json::Value;

use crate::util::Error;

#[derive(Debug, PartialEq)]
pub enum JsonNest {
    Index(usize),
    Key(String),
}

pub struct JsonPathRoot {
    nest: Vec<JsonNest>,
}

#[derive(Debug, PartialEq)]
pub enum JsonPathResult {
    String(String),
    Integer(i64),
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

    pub fn parse_value(chars: &[char]) -> Result<Vec<JsonNest>, Error> {
        if chars.len() == 0 {
            return Ok(vec![]);
        }

        match chars[0] {
            '.' => {
                let mut i = 1usize;
                loop {
                    if i >= chars.len() {
                        break;
                    }

                    if !chars[i].is_ascii_alphabetic() {
                        break;
                    }

                    i += 1;
                }

                if i <= 1 {
                    Err("Null key found".into())
                } else {
                    let key = chars[1..i].iter().collect::<String>();
                    let mut nest_rest = JsonPathRoot::parse_value(&chars[i..])?;
                    nest_rest.insert(0, JsonNest::Key(key));

                    Ok(nest_rest)
                }
            }
            '[' => {
                let mut i = 1usize;
                loop {
                    if i >= chars.len() {
                        return Err(format!("Unexpected end of index: {}", chars[i - 1]).into());
                    }

                    if chars[i] == ']' {
                        break;
                    }

                    i += 1;
                }

                if i <= 1 {
                    Err("Null index found".into())
                } else {
                    let index = usize::from_str_radix(&chars[1..i].iter().collect::<String>(), 10)
                        .expect("Invalid number");
                    let mut nest_rest = JsonPathRoot::parse_value(&chars[i + 1..])?;
                    nest_rest.insert(0, JsonNest::Index(index));

                    Ok(nest_rest)
                }
            }
            _ => Err(format!("Unexpected char: {}", chars[0]).into()),
        }
    }

    pub fn extract(&self, value: &Value) -> Result<JsonPathResult, Error> {
        let final_node = JsonPathRoot::walk_nesting(&value, &self.nest[..])?;

        if final_node.is_string() {
            Ok(JsonPathResult::String(
                final_node.as_str().unwrap().to_string(),
            ))
        } else if final_node.is_i64() {
            Ok(JsonPathResult::Integer(final_node.as_i64().unwrap()))
        } else {
            Err(format!("Unexpected JSON value type: {:?}", final_node).into())
        }
    }

    pub fn walk_nesting<'a>(value: &'a Value, nest: &[JsonNest]) -> Result<&'a Value, Error> {
        if nest.is_empty() {
            Ok(value)
        } else {
            match &nest[0] {
                JsonNest::Key(key) => {
                    if !value.is_object() {
                        Err(format!("Expected object, got: {:?}", value).into())
                    } else {
                        JsonPathRoot::walk_nesting(
                            &value
                                .as_object()
                                .expect("Walk error -> not an object")
                                .get(key)
                                .expect(&format!(
                                    "Walk error -> no value for key '{}' in {:?}",
                                    key, value
                                )),
                            &nest[1..],
                        )
                    }
                }
                JsonNest::Index(index) => {
                    if !value.is_array() {
                        Err(format!("Expected list, got: {:?}", value).into())
                    } else {
                        JsonPathRoot::walk_nesting(
                            &value
                                .as_array()
                                .expect("Walk erro -> not an array")
                                .get(*index)
                                .expect(&format!(
                                    "Walk error -> no value for index '{}' in {:?}",
                                    index, value
                                )),
                            &nest[1..],
                        )
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::Value;

    use crate::json_path::{JsonNest, JsonPathResult};

    use super::JsonPathRoot;

    #[test]
    fn test_empty() {
        let root = JsonPathRoot::from("$").unwrap();
        assert_eq!(0, root.nest.len());
    }

    #[test]
    fn test_key() {
        let root = JsonPathRoot::from("$.foo").unwrap();
        assert_eq!(1, root.nest.len());
        assert_eq!(JsonNest::Key("foo".to_string()), root.nest[0]);
    }

    #[test]
    fn test_index() {
        let root = JsonPathRoot::from("$[2]").unwrap();
        assert_eq!(1, root.nest.len());
        assert_eq!(JsonNest::Index(2), root.nest[0]);
    }

    #[test]
    fn test_complex() {
        let root = JsonPathRoot::from("$.foo.bar[0][1].baz").unwrap();
        assert_eq!(5, root.nest.len());
        assert_eq!(JsonNest::Key("foo".to_string()), root.nest[0]);
        assert_eq!(JsonNest::Key("bar".to_string()), root.nest[1]);
        assert_eq!(JsonNest::Index(0), root.nest[2]);
        assert_eq!(JsonNest::Index(1), root.nest[3]);
        assert_eq!(JsonNest::Key("baz".to_string()), root.nest[4]);
    }

    #[test]
    fn extract_empty() {
        let root = JsonPathRoot::from("$").unwrap();
        let json: Value = serde_json::from_str("12").unwrap();
        let result = root.extract(&json).unwrap();

        assert_eq!(JsonPathResult::Integer(12), result);
    }

    #[test]
    fn extract_deep() {
        let root = JsonPathRoot::from("$.foo.bar[2][0].baz").unwrap();
        let json: Value = serde_json::from_str(
            r#"
                {
                    "foo": {
                        "bar": [
                            [],
                            [],
                            [
                                {
                                    "baz": 42,
                                    "bum": -1
                                }
                            ]
                        ]
                    }
                }
            "#,
        )
        .unwrap();
        let result = root.extract(&json).unwrap();

        assert_eq!(JsonPathResult::Integer(42), result);
    }
}
