use reqwest::blocking::Response;
use serde_json::Value;
use std::io::Read;

use crate::config::Config;

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

        info!("{:#?}", json);
    }

    fn raw_execute_graphql_operation(&self, query: &str) -> Response {
        let mut request = self.client.post(self.url.clone());

        for [key, value] in &self.headers {
            request = request.header(key, value);
        }

        let body = format!("{{ \"query\": \"{}\" }}", query);

        debug!("Body: {}", &body);
        debug!("Headers: {:?}", self.headers);
        debug!("URL: {}", self.url);

        request.body(body).send().unwrap()
    }

    pub fn fetch_live_schema(&self) -> String {
        let mut response = self.raw_execute_graphql_operation(INSPECTION_QUERY);

        let mut response_body = String::new();
        response.read_to_string(&mut response_body).unwrap();

        response_body
    }
}
