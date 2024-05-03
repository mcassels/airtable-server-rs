use airtable_api::{Airtable, Record};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use std::sync::{Arc, Mutex};
use std::time::Instant;


// TODO: caching!
// https://fasterthanli.me/articles/request-coalescing-in-async-rust#a-little-caching-can-t-hurt

#[derive(Deserialize)]
pub(crate) struct TableGetParams {
    filter_by_formula: Option<String>,
}

pub(crate) async fn get_table_handler(
    Path(table): Path<String>,
    Query(query): Query<TableGetParams>,
) -> Result<Json<Vec<Record<serde_json::Value>>>, StatusCode> {
    let airtable = Airtable::new_from_env();
    let records: Vec<Record<serde_json::Value>> = airtable
        .list_records::<serde_json::Value>(&table, None, None, query.filter_by_formula.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json::from(records))
}
