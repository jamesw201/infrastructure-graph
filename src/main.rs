#[cfg_attr(test, macro_use)] extern crate serde_json;
use rust_nom_json::*;

// DONE:
// [√] read this approach for mocking reqwest with traits: https://write.as/balrogboogie/testing-reqwest-based-clients
// [√] prove that you can write end-to-end unit tests from this file
// [√] create a trait for async services that return Result<Json, Error>  
// [√] use that trait to mock responses, enabling TDD for applications with side effects  
// [√] make sure that errors are transformed into some kind of standard Error

fn main() {
    let client = http_client::HttpClient;
    let service = ip_service::IPService::new(client);

    // settled with unwrap after failing to use ? or await: https://github.com/seanmonstar/reqwest/issues/275
    let bla = service.call_api().unwrap();
    println!("api: {:#?}", bla);

    let f = foo::Foo::new("hello");
    println!("{:?}", f);
}

// TODO: 
// [ ] read Terraform files in
// [ ] begin nom parsing on files

#[cfg(test)]
mod tests {
    use super::*;
    use http_client::HttpMethods;
    use async_trait::async_trait;
    use anyhow::Result;
    use serde_json::Value;

    struct MockHttpClient;
    #[async_trait]
    impl HttpMethods for MockHttpClient {
        async fn get(&self, url: String) -> Result<Value> {
            let data = r#"{
                "id": 42,
                "foo": "bar",
                "baz": "quux"
            }"#;
            let v: Value = serde_json::from_str(data)?;
            Ok(v)
        }
    }
    
    #[test]
    fn main_test() {
        let service = ip_service::IPService::new(MockHttpClient);
        let result = service.call_api().unwrap();
        assert_eq!(result, json!({
            "id": 42,
            "foo": "bar",
            "baz": "quux"
        }));
    }
}