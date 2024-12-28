use std::io::Read;

use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde_json::Value;

pub struct NetOps {
    client: reqwest::blocking::Client,
}

impl NetOps {
    pub fn new() -> NetOps {
        NetOps {
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn execute_graphql_operation(&self, body: String) {
        let mut r = self
            .client
            .post("https://itarato.myshopify.com/api/2024-10/graphql.json")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(
                "X-Shopify-Storefront-Access-Token",
                "7f4754b975e4b1f4e04b282a37894a9a",
            )
            .body(format!("{{ \"query\": \"{}\" }}", body))
            .send()
            .unwrap();

        let mut body = String::new();
        r.read_to_string(&mut body).unwrap();

        let json: Value = serde_json::from_str(&body).unwrap();

        info!("{:#?}", json);
    }
}
