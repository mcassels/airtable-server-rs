use airtable_api::{Airtable, Record};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

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

    // TODO: Expose an endpoint to invalidate the cache.
    // (Or re-introduce a TTL for the cache.)
    if let Some(entry) = TABLE_CACHE.get(&cache_key) {
        info!("Cache hit for {}", cache_key);
        return Ok(entry.value);
    }

    // We cannot create a shared Airtable instance because
    // it contains an http client that is not Sync.
    let airtable = Airtable::new_from_env();

    info!("Cache miss for {}. Making airtable request...", cache_key);

    let records: Vec<Record<serde_json::Value>> = airtable
        .list_records::<serde_json::Value>(&table, None, None, query.filterByFormula.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let res = Json::from(records);
    let entry = crate::TableCacheEntry { value: res.clone() };
    TABLE_CACHE.insert(cache_key, entry);

    Ok(res)
}
