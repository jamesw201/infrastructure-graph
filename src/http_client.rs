extern crate reqwest;
use async_trait::async_trait;

// the idea of this file is to be a thin trait wrapper around an http client such as reqwest:
// https://stackoverflow.com/questions/51919079/how-to-mock-external-dependencies-in-tests

#[async_trait]
pub trait HttpMethods {
    async fn get(&self, url: String) -> Result<reqwest::Response, reqwest::Error>;
}

pub struct HttpClient;

#[async_trait]
impl HttpMethods for HttpClient {
    async fn get(&self, url: String) -> Result<reqwest::Response, reqwest::Error> {
        reqwest::get(&url).await
    }
}
