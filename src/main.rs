use rust_nom_json::*;

fn main() {
    let bla = ip_service::call().unwrap();
    println!("api: {:#?}", bla);

    let f = foo::Foo::new("hello");
    println!("{:?}", f);
}
