use axum::{
    routing::{get, post},
    http::StatusCode,
    extract::{ConnectInfo, State},
    Json, Router,

};
use serde::{Deserialize};
use tracing::log::info;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Mutex;
mod handler;

#[tokio::main]
async fn main() {
    let db_handler = handler::Database {
        map : Into::into(Mutex::new(HashMap::new()))
    };
    
    let config = Config {
        db: db_handler
    };

    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/", get(root))
        .route("/send", post(send))
        .route("/get", get(get_entry))
        .with_state(config);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Alive"
}

async fn send(ConnectInfo(addr): ConnectInfo<SocketAddr>, State(conf): State<Config>, Json(input) : Json<EventInput>) -> (StatusCode, Json<String>) {
    let etype = handler::EventType::WRITE;
    let event_sender = handler::Sender {
        addr: Some(addr.ip())
    };
    
    info!("{}: event sent from {}", etype, addr);
    let event = handler::Event::new(event_sender, etype, input.key, input.value);

    if let Err(_e) = conf.db.store(&event) {
        return (StatusCode::BAD_REQUEST, Json(String::from("Internal Error")));
    } else {
        return (StatusCode::OK, Json(String::from("OK")));
    }
}

async fn get_entry(ConnectInfo(_addr): ConnectInfo<SocketAddr>, State(conf): State<Config>, Json(input) : Json<EventInput>) -> (StatusCode, Json<String>) {
    if let Ok(Some(value)) = conf.db.read(input.key) {
        return (StatusCode::OK, Json(value));
    } else {
        return (StatusCode::BAD_REQUEST, Json(String::from("Error")));
    }
}

#[derive(Deserialize)]
struct EventInput {
    key: String,
    value: Option<String>
}

#[derive(Clone)]
struct Config {
    db: handler::Database
}
