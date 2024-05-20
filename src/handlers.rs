use airtable_api::{Airtable, Record};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::time::Instant;

use crate::{TableResult, TABLE_CACHE};

#[derive(Deserialize)]
#[allow(non_snake_case)] // the query params for airtable are camelCase
pub(crate) struct TableGetParams {
    filterByFormula: Option<String>,
}

pub(crate) async fn get_table_handler(
    Path(table): Path<String>,
    Query(query): Query<TableGetParams>,
) -> Result<TableResult, StatusCode> {
    let cache_key = format!(
        "{}/{}",
        table,
        query.filterByFormula.as_deref().unwrap_or("")
    );

    // Check if the cache has the value and if it is less than 60 seconds old.
    if let Some(entry) = TABLE_CACHE.get(&cache_key) {
        if entry.timestamp.elapsed().as_secs() < 60 {
            println!("Cache hit for {}", cache_key);
            return Ok(entry.value.clone());
        }
    }

    // We cannot create a shared Airtable instance because
    // it contains an http client that is not Sync.
    let airtable = Airtable::new_from_env();
    println!("Cache miss for {}. Making airtable request...", cache_key);
    let records: Vec<Record<serde_json::Value>> = airtable
        .list_records::<serde_json::Value>(&table, None, None, query.filterByFormula.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let res = Json::from(records);
    let entry = crate::TableCacheEntry {
        value: res.clone(),
        timestamp: Instant::now(),
    };
    TABLE_CACHE.insert(cache_key, entry);
    Ok(res)
}
