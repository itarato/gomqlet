use regex::Regex;
use reqwest::blocking::Response;
use serde_json::Value;
use std::{fs::File, io::Read};

use crate::{
    config::Config,
    json_path::{JsonPathResult, JsonPathRoot},
    magic_command::MagicCommand,
};

const INSPECTION_QUERY: &'static str = "query IntrospectionQuery { __schema { queryType { name } mutationType { name } subscriptionType { name } types { ...FullType } directives { name description locations args { ...InputValue } } }}fragment FullType on __Type { kind name description fields(includeDeprecated: true) { name description args { ...InputValue } type { ...TypeRef } isDeprecated deprecationReason } inputFields { ...InputValue } interfaces { ...TypeRef } enumValues(includeDeprecated: true) { name description isDeprecated deprecationReason } possibleTypes { ...TypeRef }}fragment InputValue on __InputValue { name description type { ...TypeRef } defaultValue}fragment TypeRef on __Type { kind name ofType { kind name ofType { kind name ofType { kind name } } }}";

pub struct NetOps {
    client: reqwest::blocking::Client,
    url: String,
    headers: Vec<[String; 2]>,
}

impl NetOps {
    pub fn new(config: &Config) -> NetOps {
        NetOps {
            client: reqwest::blocking::Client::new(),
            url: config.url.clone(),
            headers: config.headers.clone(),
        }
    }

    pub fn execute_graphql_operation(&self, query: &str) {
        let query = query.replace('"', "\\\"");
        let mut response = self.raw_execute_graphql_operation(&query);

        let mut response_body = String::new();
        response.read_to_string(&mut response_body).unwrap();

        let json: Value = serde_json::from_str(&response_body).unwrap();

        info!("\x1B[95mResponse: \x1B[93m{:#?}\x1B[0m", json);
    }

    fn raw_execute_graphql_operation(&self, query: &str) -> Response {
        let mut request = self.client.post(self.url.clone());

        for [key, value] in &self.headers {
            request = request.header(key, value);
        }

        let query = self.replace_magic_values(query);
        let body = format!("{{ \"query\": \"{}\" }}", query);

        debug!("\x1B[95mBody: \x1B[94m{}\x1B[0m", &body);
        debug!("\x1B[95mHeaders: \x1B[94m{:?}\x1B[0m", self.headers);
        debug!("\x1B[95mURL: \x1B[94m{}\x1B[0m", self.url);

        request.body(body).send().unwrap()
    }

    pub fn fetch_live_schema(&self) -> String {
        let mut response = self.raw_execute_graphql_operation(INSPECTION_QUERY);

        let mut response_body = String::new();
        response.read_to_string(&mut response_body).unwrap();

        response_body
    }

    fn replace_magic_values(&self, subject: &str) -> String {
        let mut out = subject.to_string();

        let re = Regex::new("<([^>]+)>").unwrap();
        let matches = re.captures_iter(subject).collect::<Vec<_>>();

        for captures in matches.iter().rev() {
            if let Some(re_match) = captures.iter().nth(1).unwrap() {
                match MagicCommand::from(re_match.as_str()) {
                    Ok(MagicCommand::Query(query_command)) => {
                        let json_response = File::open(&query_command.file).and_then(|mut file| {
                            let mut buf = String::new();
                            file.read_to_string(&mut buf)?;

                            let buf = buf.replace('\n', "");

                            let response = self.raw_execute_graphql_operation(&buf);
                            Ok(serde_json::from_reader::<_, Value>(response).unwrap())
                        });

                        if json_response.is_err() {
                            continue;
                        }

                        let replacement = match JsonPathRoot::from(&query_command.json_path)
                            .and_then(|json_path_root| {
                                json_path_root.extract(&json_response.unwrap())
                            }) {
                            Ok(JsonPathResult::Integer(int_value)) => int_value.to_string(),
                            Ok(JsonPathResult::String(str_value)) => {
                                format!("\\\"{}\\\"", str_value)
                            }
                            Err(err) => {
                                error!("Error during magic value parsing: {}", err);
                                continue;
                            }
                        };

                        out.replace_range(
                            re_match.range().start - 1..re_match.range().end + 1,
                            &replacement,
                        );
                    }
                    Err(err) => error!("Magic value parse error: {}", err),
                };
            }
        }

        out
    }
}
