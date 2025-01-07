use crate::util::Error;

#[derive(Debug, PartialEq)]
pub struct QueryCommand {
    pub file: String,
    pub json_path: String,
}

#[derive(Debug, PartialEq)]
pub enum MagicCommand {
    Query(QueryCommand),
    RandomString(usize),
    RandomInteger((i32, i32)),
}

impl MagicCommand {
    pub fn from(raw: &str) -> Result<MagicCommand, Error> {
        let parts = raw.split("::").collect::<Vec<_>>();

        match parts[0] {
            "query" => MagicCommand::parse_query(&parts[1..]),
            "random_string" => MagicCommand::parse_random_string(&parts[1..]),
            "random_integer" => MagicCommand::parse_random_integer(&parts[1..]),
            _ => Err("Unknown command prefix".into()),
        }
    }

    fn parse_query(parts: &[&str]) -> Result<MagicCommand, Error> {
        if parts.len() != 2 {
            return Err("Query command must have 2 arguments: file + json path".into());
        }

        Ok(MagicCommand::Query(QueryCommand {
            file: parts[0].to_string(),
            json_path: parts[1].to_string(),
        }))
    }

    fn parse_random_string(parts: &[&str]) -> Result<MagicCommand, Error> {
        if parts.len() != 1 {
            return Err("Random string command must have 1 argument: length".into());
        }

        Ok(MagicCommand::RandomString(usize::from_str_radix(
            parts[0], 10,
        )?))
    }

    fn parse_random_integer(parts: &[&str]) -> Result<MagicCommand, Error> {
        if parts.len() != 2 {
            return Err(
                "Random integer command must have 2 argument: min bound + max bound".into(),
            );
        }

        Ok(MagicCommand::RandomInteger((
            i32::from_str_radix(parts[0], 10)?,
            i32::from_str_radix(parts[1], 10)?,
        )))
    }
}

#[cfg(test)]
mod test {
    use crate::magic_command::QueryCommand;

    use super::MagicCommand;

    #[test]
    fn test_query() {
        let mc = MagicCommand::from("query::file.ext::$.path").unwrap();
        assert_eq!(
            MagicCommand::Query(QueryCommand {
                file: "file.ext".to_string(),
                json_path: "$.path".to_string()
            }),
            mc
        );
    }

    #[test]
    fn test_random_string() {
        let mc = MagicCommand::from("random_string::12").unwrap();
        assert_eq!(MagicCommand::RandomString(12), mc);
    }

    #[test]
    fn test_random_integer() {
        let mc = MagicCommand::from("random_integer::10::20").unwrap();
        assert_eq!(MagicCommand::RandomInteger((10, 20)), mc);
    }
}
