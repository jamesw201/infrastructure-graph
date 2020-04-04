use rust_nom_json::*;

fn main() {
    let client = http_client::HttpClient;
    let service = ip_service::IPService::new(client);

    // settled with unwrap after failing to use ? or await: https://github.com/seanmonstar/reqwest/issues/275
    let bla = service.call_api().unwrap();
    println!("api: {:#?}", bla);

    let f = foo::Foo::new("hello");
    println!("{:?}", f);
}

// TODO: prove that you can write end-to-end unit tests from this file
// #[cfg(test)]
// mod tests {
//     use super::*;

    
//     #[test]
//     fn main_test() {
//         let mock_http_clientw;
//         assert_eq!(345+5, sum(345, 5));
//     }
// }