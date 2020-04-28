#[cfg_attr(test, macro_use)] extern crate serde_json;
use rust_nom_json::*;

mod terraform;

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

    let parser = cloud_template_parser::CloudTemplateParser::new();
    parser.handle(vec![String::from("example_files/basic.tf")]);
}

// TODO: 
// [√] read Terraform files in
// [√] begin nom parsing on files

#[cfg(test)]
mod tests {
    use super::*;
    use http_client::HttpMethods;
    use async_trait::async_trait;
    use anyhow::Result;
    use serde_json::Value;
    // use std::io::BufReader;
    // use std::fs::File;

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

    // TODO: test that library can read from an array of  BufferedReader
    // depending on the filename { bla.tfvars, bla.variables, bla.tf } we will look to either parse or create values for string interpolation.
    // #[test]
    // fn buffer_reader_test() {
    //     struct FileType<T> {
    //         filename: String,
    //         reader: BufReader<T>,
    //     };
    //     impl <T> FileType<T> {
    //         pub fn new(filename: String, reader: BufReader<T>) -> FileType<T> {
    //             FileType { filename, reader }
    //         }
    //     }

    //     // 
    //     let tf_file = File::open("fake.tf").unwrap();
    //     let tfvars_file = File::open("fake.tfvars").unwrap();
    //     let variables_file = File::open("variables.tf").unwrap();

    //     let tf = FileType::new(String::from("fake.tf"), BufReader::new(tf_file));
    //     let tfvars = FileType::new(String::from("fake.tfvars"), BufReader::new(tfvars_file));
    //     let variables = FileType::new(String::from("variables.tf"), BufReader::new(variables_file));

    //     let files = vec![tf, tfvars, variables];
    //     let parser = cloud_template_parser::CloudTemplateParser::new();
    //     let result = parser.handle(vec![String::from("discovery.tf")]);
    //     assert_eq!(true, true)

    //     // assert_eq!(result, json!({
    //     //     "resources": [
    //     //         {
    //     //             "resource": "bla",
    //     //             "foo": "bar",
    //     //             "baz": "quux",
    //     //         },
    //     //         {
    //     //             "resource": "bloop",
    //     //             "foo": "bar",
    //     //             "baz": "quux",
    //     //         }
    //     //     ],
    //     //     "relationships": [
    //     //         {
    //     //             "source": "bla",
    //     //             "target": "bloop",
    //     //         }
    //     //     ]
    //     // }));
    // }
}
