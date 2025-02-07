use regex::Regex;
use reqwest::blocking::Response;
use serde_json::Value;
use std::{fs::File, io::Read, time::Duration};

use crate::{
    config::Config,
    json_path::{JsonPathResult, JsonPathRoot},
    magic_command::MagicCommand,
    util::{err_ctx, random_integer, random_string, random_word, Error},
};

const INSPECTION_QUERY: &'static str = "query IntrospectionQuery { __schema { queryType { name } mutationType { name } subscriptionType { name } types { ...FullType } directives { name description locations args { ...InputValue } } }}fragment FullType on __Type { kind name description fields(includeDeprecated: true) { name description args { ...InputValue } type { ...TypeRef } isDeprecated deprecationReason } inputFields { ...InputValue } interfaces { ...TypeRef } enumValues(includeDeprecated: true) { name description isDeprecated deprecationReason } possibleTypes { ...TypeRef }}fragment InputValue on __InputValue { name description type { ...TypeRef } defaultValue}fragment TypeRef on __Type { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name } } } } }}";
const TIMEOUT_SECONDS: u64 = 120;

pub struct NetOps {
    client: reqwest::blocking::Client,
    url: String,
    headers: Vec<[String; 2]>,
    variables: Option<Value>,
}

impl NetOps {
    pub fn new(config: &Config) -> NetOps {
        NetOps {
            client: reqwest::blocking::Client::new(),
            url: config.url.clone(),
            headers: config.headers.clone(),
            variables: config.variables.clone(),
        }
    }

    pub fn execute_graphql_operation(&self, query: &str) {
        let query = query.replace('"', "\\\"");
        let response = match self.raw_execute_graphql_operation(&query) {
            Ok(response) => response,
            Err(err) => {
                error!("Error while executing query over HTTP: {}", err);
                return;
            }
        };
        
        let _ = serde_json::from_reader(response).map(|json: Value| {
            info!("\x1B[95mResponse: \x1B[93m{:#?}\x1B[0m", json);
        }).map_err(|e| {
            error!("Failed to deserialize JSON respone: {}", e);
        });
    }

    fn raw_execute_graphql_operation(&self, query: &str) -> Result<Response, Error> {
        let mut request = self
            .client
            .post(&self.url)
            .timeout(Duration::from_secs(TIMEOUT_SECONDS));

        for [key, value] in &self.headers {
            request = request.header(key, value);
        }

        let query = self
            .replace_magic_values(query)
            .map_err(err_ctx("Failed query execution"))?;
        let body = format!("{{ \"query\": \"{}\" }}", query);

        debug!("\x1B[95mBody: \x1B[94m{}\x1B[0m", &body);
        debug!("\x1B[95mHeaders: \x1B[94m{:?}\x1B[0m", self.headers);
        debug!("\x1B[95mURL: \x1B[94m{}\x1B[0m", self.url);

        Ok(request.body(body).send().unwrap())
    }

    pub fn fetch_live_schema(&self) -> Result<String, Error> {
        let mut response = self.raw_execute_graphql_operation(INSPECTION_QUERY)?;

        let mut response_body = String::new();
        response.read_to_string(&mut response_body).unwrap();

        Ok(response_body)
    }

    fn replace_magic_values(&self, subject: &str) -> Result<String, Error> {
        let mut out = subject.to_string();

        let re = Regex::new("<([^>]+)>").unwrap();
        let matches = re.captures_iter(subject).collect::<Vec<_>>();

        for captures in matches.iter().rev() {
            if let Some(re_match) = captures.iter().nth(1).unwrap() {
                let magic_command = MagicCommand::from(re_match.as_str())
                    .map_err(err_ctx("Failed magic value interpretation"))?;

                debug!(
                    "\x1B[95mReplacing magic command: \x1B[92m{:?}\x1B[0m",
                    &magic_command
                );

                let replacement = match magic_command {
                    MagicCommand::Query(query_command) => {
                        let json_response = File::open(&query_command.file)
                            .map_err(|io_error| {
                                format!("File cannot be opened: {}", io_error).into()
                            })
                            .and_then(|mut file| {
                                let mut buf = String::new();
                                file.read_to_string(&mut buf)?;

                                let buf = buf.replace('\n', "");

                                self.raw_execute_graphql_operation(&buf)
                            })
                            .and_then(|response| {
                                Ok(serde_json::from_reader::<_, Value>(response).unwrap())
                            })?;

                        JsonPathRoot::from(&query_command.json_path)
                            .and_then(|json_path_root| json_path_root.extract(&json_response))
                            .map(|json_path_result| {
                                NetOps::insertable_snippet_from_json_path_result(json_path_result)
                            })?
                    }
                    MagicCommand::RandomInteger((min, max)) => random_integer(min, max).to_string(),
                    MagicCommand::RandomString(len) => {
                        format!("\\\"{}\\\"", random_string(len))
                    }
                    MagicCommand::RandomWord => format!("\\\"{}\\\"", random_word()),
                    MagicCommand::Variable(json_path_root) => self
                        .variables
                        .as_ref()
                        .ok_or("Variables are not defined in config json".into())
                        .and_then(|variables: _| json_path_root.extract(&variables))
                        .map(|result| NetOps::insertable_snippet_from_json_path_result(result))?,
                };

                debug!("\x1B[95mReplacement: \x1B[92m{}\x1B[0m", replacement);

                out.replace_range(
                    re_match.range().start - 1..re_match.range().end + 1,
                    &replacement,
                );
            }
        }

        Ok(out)
    }

    fn insertable_snippet_from_json_path_result(json_path_result: JsonPathResult) -> String {
        match json_path_result {
            JsonPathResult::Integer(int_value) => int_value.to_string(),
            JsonPathResult::String(str_value) => {
                format!("\\\"{}\\\"", str_value)
            }
        }
    }
}
