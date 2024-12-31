use std::{fs::File, io::Read};

use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde_json::Value;

use crate::CommandLineParams;

pub struct NetOps {
    client: reqwest::blocking::Client,
    url: String,
    headers: Vec<[String; 2]>,
}

impl NetOps {
    pub fn new(command_line_params: &CommandLineParams) -> NetOps {
        

        NetOps {
            client: reqwest::blocking::Client::new(),
            url: command_line_params.
        }
    }

    pub fn execute_graphql_operation(&self, body: String) {
        let mut r = self
            .client
            .post("https://<ADD-DOMAIN>/api/2024-10/graphql.json")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header("X-Shopify-Storefront-Access-Token", "<ADD-KEY>")
            .body(format!("{{ \"query\": \"{}\" }}", body))
            .send()
            .unwrap();

        let mut body = String::new();
        r.read_to_string(&mut body).unwrap();

        let json: Value = serde_json::from_str(&body).unwrap();

        info!("{:#?}", json);
    }
}
