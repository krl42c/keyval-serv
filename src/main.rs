use axum::{
    routing::{get, post},
    http::StatusCode,
    extract::{ConnectInfo, State},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::log::{info, warn, error};
use std::net::SocketAddr;
use dotenv::dotenv;
mod handler;
use mongodb::{ bson::doc, bson::Document, options::{ ClientOptions, ServerApiVersion }, Client };

#[tokio::main]
async fn main() {
    dotenv().ok();
   
    tracing_subscriber::fmt::init();
    let mongo_uri = std::env::var("MONGO_URL").expect("EMPTY");
    let client = Client::with_uri_str(mongo_uri)
        .await
        .unwrap();

    let res = client
        .database("sample_weatherdata")
        .run_command(doc! {"ping": 1}, None)
        .await;

    match res {
        Ok(ref _client) => {
            info!("Successfully connected to MongoDB cluster");
        },
        Err(ref _client) => {
            error!("Could not establish connection with MongoDB cluster, shutting down");
            panic!();
        }
    }

    let app = Router::new()
        .route("/health", get(root))
        .route("/send", post(send))
        .route("/get", get(get_entry))
        .with_state(client);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Configuration finished, server started");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

async fn root(State(mongo) : State<Client>) -> Json<HealthCheck> {
    let res = mongo
        .database("sample")
        .run_command(doc! {"ping": 1}, None)
        .await;
    let status = HealthCheck { 
        server: Status::OK,
        mongo: match res {
            Ok(..) => Status::OK,
            Err(..) => Status::KO
        }
    };

    return Json(status);
}

async fn send(ConnectInfo(addr): ConnectInfo<SocketAddr>, State(mongo): State<Client>, Json(input) : Json<EventInput>) -> (StatusCode, Json<Vec<Option<SampleObject>>>) {
    let q = mongo
        .database("sample");
    let col = q.collection::<SampleObject>("data");

    let res = col.find_one(doc!{"object.id" : &input.key}, None).await.unwrap();
    match res {
        Some(..) => {
            let mut vec = Vec::new();
            vec.push(res);
            info!("mongodb: read: {} ", &input.key);
            return (StatusCode::OK, Json(vec))
        }
        None => {
            info!("mongodb: document {} not found", &input.key);
            return (StatusCode::NOT_FOUND, Json(Vec::new()))
        }
    }
}

async fn get_entry(ConnectInfo(_addr): ConnectInfo<SocketAddr>, State(mongo): State<Client>, Json(input) : Json<EventInput>) -> (StatusCode, Json<String>) {
    return (StatusCode::OK, Json(String::from("ALL OK")))
}

#[derive(Deserialize)]
struct EventInput {
    key: String,
    value: Option<String>
}

#[derive(Clone)]
struct Config {
    mongo_client: Client,
    db: handler::Database
}

#[derive(Serialize)]
enum Status {
    OK,
    KO
}

#[derive(Serialize)]
struct HealthCheck {
    server: Status,
    mongo: Status
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SampleObject {
    object: Object    
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Object {
    sample_data: String,
    id: String
}

