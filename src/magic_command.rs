pub struct QueryCommand {
    pub file: String,
    pub xpath: String,
}

pub enum MagicCommand {
    Query(QueryCommand),
}

impl MagicCommand {
    pub fn from(raw: &str) -> Result<MagicCommand, String> {
        let parts = raw.split("::").collect::<Vec<_>>();

        match parts[0] {
            "query" => MagicCommand::parse_query(&parts[1..]),
            _ => Err("Unknown command prefix".to_string()),
        }
    }

    fn parse_query(parts: &[&str]) -> Result<MagicCommand, String> {
        if parts.len() != 2 {
            return Err("Query command must have 2 arguments: file + xpath".to_string());
        }

        Ok(MagicCommand::Query(QueryCommand {
            file: parts[0].to_string(),
            xpath: parts[1].to_string(),
        }))
    }
}
