use std::net::SocketAddr;

use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router, Server,
};
use clap::Parser;
use html_template::HtmlTemplate;
use person::{CreatePersonDto, Person};

mod html_template;

#[derive(Debug, Clone, Parser)]
struct Args {
    /// The address to listen on
    #[clap(long, env)]
    pub address: String,

    /// The port to listen on
    #[clap(long, env)]
    pub port: u16,

    /// The url of the person service
    #[clap(long, env)]
    pub person_url: String,
}

#[derive(Template)]
#[template(path = "persons.html")]
struct PersonsTemplate {
    pub persons: Vec<Person>,
    pub page_title: String,
}

#[derive(Template)]
#[template(path = "person.html")]
struct PersonTemplate {
    pub person: Person,
}

async fn create_person(
    person_address: String,
    person: CreatePersonDto,
) -> Result<Person, reqwest::Error> {
    let client = reqwest::Client::new();
    let result = client
        .post(&format!("{}/persons", person_address))
        .json(&person)
        .send()
        .await?
        .json::<Person>()
        .await?;

    Ok(result)
}

async fn fetch_persons(person_address: String) -> Vec<Person> {
    reqwest::get(&format!("{}/persons", person_address))
        .await
        .unwrap()
        .json::<Vec<Person>>()
        .await
        .unwrap()
}

async fn get_persons_handler(State(state): State<Args>) -> impl IntoResponse {
    let template = PersonsTemplate {
        persons: fetch_persons(state.person_url.clone()).await,
        page_title: "Person".to_string(),
    };

    HtmlTemplate(template)
}

async fn create_person_handler(
    State(state): State<Args>,
    Json(person): Json<CreatePersonDto>,
) -> impl IntoResponse {
    let person = create_person(state.person_url.clone(), person)
        .await
        .unwrap();

    let template = PersonTemplate { person };

    HtmlTemplate(template)
}

async fn live() -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok((StatusCode::OK, "OK"))
}

async fn ready(State(state): State<Args>) -> impl IntoResponse {
    match reqwest::get(&format!("{}/health/ready", state.person_url)).await {
        Ok(_) => (StatusCode::OK, "OK"),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable"),
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let router = Router::new()
        .route("/persons", get(get_persons_handler))
        .route("/persons", post(create_person_handler))
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .with_state(args.clone());

    println!("listening on {}:{}", args.address, args.port);
    let socker_addr: SocketAddr = format!("{}:{}", args.address, args.port).parse().unwrap();
    Server::bind(&socker_addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
