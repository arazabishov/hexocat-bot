#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate serde;
extern crate serde_json;
extern crate rocket;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate anterofit;

use std::io::Cursor;
use anterofit::{Adapter, Url};
use anterofit::net::intercept::AddHeader;
use useragent::UserAgentHeader;
use rocket::request::LenientForm;
use rocket::response::Response;
use rocket::http::{ContentType, Status};
use rocket::config::{self, ConfigError, Environment};

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

#[derive(Debug, Serialize)]
struct SlackResponse {
    text: String,
    response_type: String,
}

#[derive(FromForm)]
struct SlackRequest {
    text: String,
    token: String
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
        .map(|repo| format!("<{0}|{1}> by <{2}|{3}>\n{4}\n----", repo.html_url,
                repo.name, repo.owner.html_url, repo.owner.login, repo.description))
        .collect::<Vec<String>>()
        .join("\n\n");
}

fn prepare_response(text: String) -> Response<'static> {
    let body = serde_json::to_string(&SlackResponse {
        text: text,
        response_type: "in_channel".to_string()
    }).unwrap();

    return Response::build()
        .status(Status::Ok)
        .header(ContentType::JSON)
        .sized_body(Cursor::new(body))
        .finalize();
}

#[post("/", data = "<form_request>")]
fn hexocat(form_request: LenientForm<SlackRequest>) -> Response<'static> {
    let request = form_request.get();

    if request.text.trim().is_empty() {
        return prepare_response("Specify repository name to search. \
                For example: /hexocat linux".to_string());
    }

    let service = Adapter::builder()
        .base_url(Url::parse("https://api.github.com").unwrap())
        .interceptor(AddHeader(UserAgentHeader("hexocat-bot".to_string())))
        .serialize_json()
        .build();

    let repository = request.text.to_lowercase().to_string();
    let response_body = match service.search(repository, 10).exec().block() {
        Ok(result) => prepare_response_body(result.items),
        Err(_) => "Oops, something went wrong.".to_string()
    };

    return prepare_response(response_body);
}

fn main() {
    rocket::ignite().mount("/hexocat/", routes![hexocat]).launch();
}
