extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
struct Owner {
    login: String,
    html_url: String
}

#[derive(Deserialize)]
struct Repository {
    name: String,
    html_url: String,
    description: String,
    owner: Owner
}

#[derive(Deserialize)]
struct SearchResult {
    items: Vec<Repository>
}

fn main() {
    println!("Hello, world!");
}
