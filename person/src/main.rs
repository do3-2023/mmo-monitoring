use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use clap::Parser;
use mysql::{Pool, PooledConn};
use person::{CreatePersonDto, Person};

use mysql::prelude::Queryable;

#[derive(Debug, Clone)]
struct AppState {
    pub pool: Pool,
}

impl AppState {
    pub fn new(pool: Pool) -> Self {
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

    /// The mysql database host
    #[clap(long, env)]
    pub mysql_host: String,

    /// The mysql database port
    #[clap(long, env)]
    pub mysql_port: u16,

    /// The mysql database user
    #[clap(long, env)]
    pub mysql_user: String,

    /// The mysql database password
    #[clap(long, env)]
    pub mysql_password: String,

    /// The mysql database name
    #[clap(long, env)]
    pub mysql_dbname: String,
}

#[derive(Debug, Clone)]
struct PersonError {
    message: String,
}

impl PersonError {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

async fn create_person(
    conn: &mut PooledConn,
    person: CreatePersonDto,
) -> Result<Person, PersonError> {
    let query = "INSERT INTO person (last_name, phone_number, location)
        VALUES (:last_name, :phone_number, :location)";
    conn.exec_drop(
        query,
        (&person.last_name, &person.phone_number, &person.location),
    )
    .map_err(|e| PersonError::new(&format!("Failed to run insert query: {}", e)))?;

    Ok(Person {
        id: 0,
        last_name: person.last_name,
        phone_number: person.phone_number,
        location: person.location,
    })
}

async fn list_persons(conn: &mut PooledConn) -> Result<Vec<Person>, PersonError> {
    let query = "SELECT id, last_name, phone_number, location FROM person";
    let persons = conn
        .query_map(query, |(id, last_name, phone_number, location)| Person {
            id,
            last_name,
            phone_number,
            location,
        })
        .map_err(|e| PersonError::new(&format!("Failed to run select query: {}", e)))?;

    Ok(persons)
}

async fn get_persons_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = &mut state.pool.get_conn().unwrap();
    match list_persons(conn).await {
        Ok(persons) => Ok((StatusCode::OK, Json(persons))),
        Err(e) => {
            println!("Error: {}", e.message);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ))
        }
    }
}

async fn create_person_handler(
    State(state): State<AppState>,
    Json(person): Json<CreatePersonDto>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = &mut state.pool.get_conn().unwrap();
    match create_person(conn, person).await {
        Ok(person) => Ok((StatusCode::CREATED, Json(person))),
        Err(e) => {
            println!("Error: {}", e.message);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ))
        }
    }
}

async fn live() -> impl IntoResponse {
    StatusCode::OK
}

async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let conn = &mut state.pool.get_conn().unwrap();
    let query = "SELECT 1";
    match conn.query_drop(query) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

use tokio::net::TcpListener;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        args.mysql_user, args.mysql_password, args.mysql_host, args.mysql_port, args.mysql_dbname
    );
    let pool = Pool::new(url.as_str()).unwrap();

    // build our application with a route
    let app = Router::new()
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .route(
            "/persons",
            get(get_persons_handler).post(create_person_handler),
        )
        .with_state(AppState::new(pool));

    // run it
    let addr = format!("{}:{}", args.address, args.port);
    let tcp_listener = TcpListener::bind(addr.clone()).await.unwrap();
    println!("listening on {}", addr);
    axum::Server::from_tcp(tcp_listener.into_std().unwrap())
        .unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();
}
