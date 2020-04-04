use std::collections::HashMap;

use crate::http_client;
use http_client::{ HttpMethods };

// a good resource on Rust modules: https://dev.to/saiumesh/modules-in-rust-programming-language-495m

// TODO: complete the object oriented chapters of the rust book, they probably confirm what to do about class like structures.
// DONE: turn this function into an api_caller struct with method 'call'
// The struct constructor should receive an http_client trait object as a dependency

// TODO: document on the idea of 'inverting the dependency tree' when composing components at the entry point to the application.

pub struct IPService<T> {
    client: T,
}

// struct impl with trait explained: https://users.rust-lang.org/t/using-any-struct-that-implements-a-trait-in-another-struct/13474/4 
impl <T> IPService<T> 
    where T: HttpMethods {

    pub fn new(client: T) -> IPService<T> {
        IPService { client }
    }

    #[tokio::main]
    pub async fn call_api(&self) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let resp = self.client.get(String::from("https://httpbin.org/ip"))
            .await?
            .json::<HashMap<String,     String>>()
            .await?;

        Ok(resp)
    }
}
