use rust_nom_json::*;

fn main() {
    // settled with unwrap after failing to use ? or await: https://github.com/seanmonstar/reqwest/issues/275
    let bla = ip_service::call().unwrap();
    println!("api: {:#?}", bla);

    let f = foo::Foo::new("hello");
    println!("{:?}", f);
}
