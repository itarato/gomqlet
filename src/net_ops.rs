use serde_json::Value;
use std::io::Read;

use crate::config::Config;

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

    pub fn execute_graphql_operation(&self, query: String) {
        let mut request = self.client.post(self.url.clone());

        for [key, value] in &self.headers {
            request = request.header(key, value);
        }

        let query = query.replace('"', "\\\"");
        let body = format!("{{ \"query\": \"{}\" }}", query);

        debug!("Body: {}", &body);
        debug!("Headers: {:?}", self.headers);
        debug!("URL: {}", self.url);

        let mut response = request.body(body).send().unwrap();

        let mut response_body = String::new();
        response.read_to_string(&mut response_body).unwrap();

        let json: Value = serde_json::from_str(&response_body).unwrap();

        info!("{:#?}", json);
    }
}
