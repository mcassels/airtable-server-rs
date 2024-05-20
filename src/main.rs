use airtable_api::Record;
use axum::{routing::get, Json, Router};
use dotenv::dotenv;
use lazy_static::lazy_static;
use mini_moka::sync::Cache;
use std::sync::Arc;

mod handlers;
use handlers::get_table_handler;

pub(crate) type TableResult = Json<Vec<Record<serde_json::Value>>>;

#[derive(Clone)]
pub(crate) struct TableCacheEntry {
    pub(crate) value: TableResult,
    pub(crate) timestamp: std::time::Instant,
}

lazy_static! {
    static ref TABLE_CACHE: Arc<Cache<String, TableCacheEntry>> = Arc::new(Cache::new(10000));
}

#[tokio::main]
async fn main() {
    // Load the .env file into the environment.
    dotenv().ok();

    let app = Router::new().route("/table/:id", get(get_table_handler));

    // fly.io requires this port
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
