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

use rocket::State;
use rocket::fairing::AdHoc;
use rocket::response::Response;
use rocket::config::Environment;
use rocket::request::LenientForm;
use rocket::http::{ContentType, Status};

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
    description: Option<String>,
    owner: Owner
}

#[derive(Deserialize)]
struct SearchResult {
    items: Vec<Repository>
}

// Encapsulates response body which is sent to
// Slack after searching repositories on GitHub.
#[derive(Serialize)]
struct SlackResponse {
    text: String,
    response_type: String,
}

// Struct that contains properties
// which Slack sends to the hexocat-bot.
#[derive(FromForm)]
struct SlackRequest {
    text: String,
    token: String
}

// Encapsulates useful properties which are
// provided by Rocket on start (see main function).
struct Configuration {
    // Enum value which represents Environment
    // type: Development, Staging, or Production.
    environment: Environment,

    // Token that is used to verify Slack
    // requests coming from the internet.
    // This property is used only on Staging and
    // Production environments.
    token: String
}

// Calling Anterofit's service macro.
service! {
    trait GitHubService {
        // First parameter matches to the search keyword, while
        // the second one to the search results page size. Returns
        // an instance of SearchResult struct.
        fn search(&self, q: String, p: u32) -> SearchResult {
            GET("/search/repositories");
            query!{ "q" => q, "per_page" => p }
        }
    }
}

// Takes in a vector of Repositories and formats them
// using markdown supported by Slack.
fn prepare_response_body(repos: Vec<Repository>) -> String {
    return repos.iter()
        .map(|repo| format!("<{0}|{1}> by <{2}|{3}>\n{4}\n----", repo.html_url,
                repo.name, repo.owner.html_url, repo.owner.login, repo.description.as_ref().unwrap_or(&"-".to_string())))
        .collect::<Vec<String>>()
        .join("\n\n");
}

// Accepts a message body and embeds it into the SlackResponse
// instance. The latter one is wrapped into Response and
// returned to the caller.
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

// Returns true when running in the development environment
// or when incoming token matches the one in the configuration.
fn check_access(config: &Configuration, token: String) -> bool {
    return match config.environment {
        Environment::Development => true,
        Environment::Staging => config.token.eq(&token),
        Environment::Production => config.token.eq(&token)
    };
}

// The 'allow' attribute here is just to silence the
// Rocket warning. The State<Configuration> is managed
// properly during rocket start.
#[allow(unmanaged_state)]
#[post("/", data = "<request>")]
fn hexocat(request: LenientForm<SlackRequest>, s: State<Configuration>) -> Response<'static> {
    // Extract the SlackRequest instance
    // from the LenientForm wrapper.
    let slack_request = request.get();

    // If access is not granted, return 403.
    if !check_access(s.inner(), slack_request.token.to_owned()) {
        return Response::build()
            .status(Status::Forbidden)
            .finalize();
    }

    // In case if Slack request doesn't contain a search query,
    // return a meaningful message back to the user.
    if slack_request.text.trim().is_empty() {
        return prepare_response("Specify repository name to search. \
                For example: /hexocat linux".to_string());
    }

    // Construct an instance of the GitHub service.
    let service = Adapter::builder()
        .base_url(Url::parse("https://api.github.com").unwrap())
        .interceptor(AddHeader(UserAgentHeader("hexocat-bot".to_string())))
        .serialize_json()
        .build();

    // Query GitHub API and transform the response into human readable message.
    let repository = slack_request.text.to_lowercase().to_string();
    let response_body = match service.search(repository, 10).exec().block() {
        Ok(result) => prepare_response_body(result.items),
        Err(_) => "Oops, something went wrong.".to_string()
    };

    // Wrap the message into response and
    // return it to the client.
    return prepare_response(response_body);
}

fn main() {
    rocket::ignite()
        .attach(AdHoc::on_attach(|rocket| {
            // Rocket Config is passed here as a part
            // of the application state.
            // It does contain properties which we
            // will use later, like environment and token.
            let config = rocket.config().clone();
            return Ok(rocket.manage(Configuration {
                environment: config.environment,
                token: config.get_str("key").unwrap_or("").to_string()
            }));
        }))
        .mount("/hexocat/", routes![hexocat])
        .launch();
}
