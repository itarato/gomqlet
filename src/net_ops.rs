use std::io::Read;

use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde_json::Value;

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

    pub fn execute_graphql_operation(&self, body: String) {
        let mut request = self.client.post(self.url.clone());

        for [key, value] in &self.headers {
            request = request.header(key, value);
        }

        let mut response = request
            .body(format!("{{ \"query\": \"{}\" }}", body))
            .send()
            .unwrap();

        let mut body = String::new();
        response.read_to_string(&mut body).unwrap();

        let json: Value = serde_json::from_str(&body).unwrap();

        info!("{:#?}", json);
    }
}
