use std::collections::HashMap;

use crate::http_client;
use http_client::HttpMethods;
// a good resource on Rust modules: https://dev.to/saiumesh/modules-in-rust-programming-language-495m

// TODO: turn this function into an api_caller struct with method 'call'
// The struct constructor should receive an http_client trait object as a dependency
#[tokio::main]
pub async fn call() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let resp = http_client::HttpClient::get(String::from("https://httpbin.org/ip"))
        .await?
        .json::<HashMap<String, String>>()
        .await?;

    Ok(resp)
}

// pub async fn call() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
//     let resp = reqwest::get("https://httpbin.org/ip")
//         .await?
//         .json::<HashMap<String, String>>()
//         .await?;

//     Ok(resp)
// }
