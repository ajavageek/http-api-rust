use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use config::{Config, ConfigError, Environment};
use deadpool_postgres::{Config as DPConfig, Pool};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_postgres::row::Row;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let app_state = AppState::create().await;
    let app = Router::new()
        .route("/persons", get(get_all))
        .route("/persons/:id", get(get_by_id))
        .with_state(Arc::clone(&app_state));
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct ConfigBuilder {
    postgres: DPConfig,
}

impl ConfigBuilder {
    async fn from_env() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(
                Environment::with_prefix("POSTGRES")
                    .separator("_")
                    .keep_prefix(true)
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()
    }
}

struct AppState {
    pool: Pool,
}

impl AppState {
    async fn create() -> Arc<AppState> {
        let cfg_builder = ConfigBuilder::from_env().await.unwrap();
        let pool = cfg_builder
            .postgres
            .create_pool(
                Some(deadpool_postgres::Runtime::Tokio1),
                tokio_postgres::NoTls,
            )
            .unwrap();
        Arc::new(AppState { pool })
    }
}

async fn get_all(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let client = state.pool.get().await.unwrap();
    let rows = client
        .query("SELECT id, first_name, last_name FROM person", &[])
        .await
        .unwrap();
    let mut persons: Vec<Person> = Vec::new();
    for row in rows.iter() {
        persons.push(row.into());
    }
    (StatusCode::OK, Json(persons))
}

async fn get_by_id(Path(id): Path<Uuid>, State(state): State<Arc<AppState>>) -> Response {
    let client = state.pool.get().await.unwrap();
    let rows = client
        .query(
            "SELECT id, first_name, last_name FROM person WHERE id=$1",
            &[&id],
        )
        .await
        .unwrap();
    match rows.first() {
        Some(row) => {
            let person: Person = row.into();
            (StatusCode::OK, Json(person)).into_response()
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

#[derive(Serialize)]
struct Person {
    id: Uuid,
    first_name: String,
    last_name: String,
}

impl From<&Row> for Person {
    fn from(row: &Row) -> Self {
        let id: Uuid = row.get("id");
        let first_name: String = row.get("first_name");
        let last_name: String = row.get("last_name");
        Person {
            id,
            first_name,
            last_name,
        }
    }
}
