use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::SocketAddr;

#[derive(Debug, Clone)]
struct AppState {
    pub pool: PgPool,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The address to listen on
    #[clap(long, env)]
    pub address: String,

    /// The port to listen on
    #[clap(long, env)]
    pub port: u16,

    /// The postgres database host
    #[clap(long, env)]
    pub postgres_host: String,

    /// The postgres database port
    #[clap(long, env)]
    pub postgres_port: u16,

    /// The postgres database user
    #[clap(long, env)]
    pub postgres_user: String,

    /// The postgres database password
    #[clap(long, env)]
    pub postgres_password: String,

    /// The postgres database name
    #[clap(long, env)]
    pub postgres_dbname: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    id: i32,
    last_name: String,
    phone_number: String,
    location: String,
}

async fn create_person(pool: &PgPool, person: Person) -> Result<Person, sqlx::Error> {
    let person = sqlx::query_as!(
        Person,
        "INSERT INTO person (last_name, phone_number, location) VALUES ($1, $2, $3) RETURNING *",
        person.last_name,
        person.phone_number,
        person.location,
    )
    .fetch_one(pool)
    .await?;
    Ok(person)
}

async fn list_persons(pool: &PgPool) -> Result<Vec<Person>, sqlx::Error> {
    let persons = sqlx::query_as!(Person, "SELECT * FROM person")
        .fetch_all(pool)
        .await?;
    Ok(persons)
}

async fn get_persons_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match list_persons(&state.pool).await {
        Ok(persons) => Ok((StatusCode::OK, Json(persons))),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )),
    }
}

async fn create_person_handler(
    State(state): State<AppState>,
    Json(person): Json<Person>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match create_person(&state.pool, person).await {
        Ok(person) => Ok((StatusCode::CREATED, Json(person))),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )),
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let args = Args::parse();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&format!(
            "postgres://{}:{}@{}:{}/{}",
            args.postgres_user,
            args.postgres_password,
            args.postgres_host,
            args.postgres_port,
            args.postgres_dbname,
        ))
        .await?;

    let app_state = AppState::new(pool);
    let router = Router::new()
        .route("/persons", get(get_persons_handler))
        .route("/persons", post(create_person_handler))
        .with_state(app_state);

    let socket_addr: SocketAddr = format!("{}:{}", args.address, args.port).parse().unwrap();
    axum::Server::bind(&socket_addr)
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}