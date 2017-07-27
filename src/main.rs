extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate anterofit;

use std::env;
use anterofit::{Adapter, Url};
use anterofit::net::intercept::AddHeader;
use useragent::UserAgentHeader;

mod useragent;

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

service! {
    trait GitHubService {
        fn search(&self, q: String, p: u32) -> SearchResult {
            GET("/search/repositories");
            query!{ "q" => q, "per_page" => p }
        }
    }
}

fn prepare_response_body(repos: Vec<Repository>) -> String {
    return repos.iter()
        .map(|repo| format!("{0} by {1}: {2}",
            repo.name, repo.owner.login, repo.html_url))
        .collect::<Vec<String>>()
        .join("\n");
}

fn main() {
    // When running app through cargo, the first argument
    // is a path to the binary being executed. Hence, if repository
    // name is provided, the argument count must be at least two.
    if env::args().count() < 2 {
        println!("Please, specify repository name you would like to find.");
        return;
    }

    let service = Adapter::builder()
        .base_url(Url::parse("https://api.github.com").unwrap())
        .interceptor(AddHeader(UserAgentHeader("hexocat-bot".to_string())))
        .serialize_json()
        .build();

    let repository = env::args().last().unwrap();
    let response = match service.search(repository.to_string(), 10).exec().block() {
        Ok(result) => prepare_response_body(result.items),
        Err(error) => "Oops, something went wrong.".to_string()
    };

    println!("{}", response);
}
