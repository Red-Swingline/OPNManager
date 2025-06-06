use crate::db::Database;
use crate::http_client::make_http_request;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Snapshot {
    pub uuid: String,
    pub name: String,
    pub active: String,
    pub mountpoint: String,
    pub size: String,
    pub created_str: String,
    pub created: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotSearchResponse {
    pub total: u32,
    pub rowCount: u32,
    pub current: u32,
    pub rows: Vec<Snapshot>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewSnapshotResponse {
    pub name: String,
    pub uuid: String,
}

fn build_api_url(api_info: &crate::db::ApiInfo, endpoint: &str) -> String {
    format!("{}:{}{}", api_info.api_url, api_info.port, endpoint)
}

#[tauri::command]
pub async fn is_snapshots_supported(database: State<'_, Database>) -> Result<bool, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/core/snapshots/is_supported/");

    let response = make_http_request(
        "GET",
        &url,
        None,
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let result = response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if let Some(supported) = result.get("supported").and_then(|v| v.as_bool()) {
        Ok(supported)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn get_snapshots(
    current_page: u32,
    rows_per_page: u32,
    database: State<'_, Database>,
) -> Result<SnapshotSearchResponse, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/core/snapshots/search");

    let payload = json!({
        "current": current_page,
        "rowCount": rows_per_page,
        "sort": {},
        "searchPhrase": ""
    });

    let response = make_http_request(
        "POST",
        &url,
        Some(payload),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<SnapshotSearchResponse>()
        .await
        .map_err(|e| format!("Failed to parse snapshots: {}", e))
}

#[tauri::command]
pub async fn get_new_snapshot(
    database: State<'_, Database>,
) -> Result<NewSnapshotResponse, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/core/snapshots/get/");

    let response = make_http_request(
        "GET",
        &url,
        None,
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<NewSnapshotResponse>()
        .await
        .map_err(|e| format!("Failed to get new snapshot info: {}", e))
}

#[tauri::command]
pub async fn get_snapshot(
    uuid: String,
    fetch_mode: Option<String>,
    database: State<'_, Database>,
) -> Result<Snapshot, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let mut url = build_api_url(&api_info, &format!("/api/core/snapshots/get/{}", uuid));

    if let Some(mode) = fetch_mode {
        url = format!("{}?fetchmode={}", url, mode);
    }

    let response = make_http_request(
        "GET",
        &url,
        None,
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<Snapshot>()
        .await
        .map_err(|e| format!("Failed to get snapshot: {}", e))
}

#[tauri::command]
pub async fn add_snapshot(
    name: String,
    uuid: Option<String>,
    database: State<'_, Database>,
) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/core/snapshots/add/");

    let payload = match uuid {
        Some(id) => json!({
            "uuid": id,
            "name": name
        }),
        None => json!({
            "uuid": "",
            "name": name
        }),
    };

    info!("Creating snapshot with payload: {:?}", payload);

    let response = make_http_request(
        "POST",
        &url,
        Some(payload),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to add snapshot: {}", e))
}

#[tauri::command]
pub async fn delete_snapshot(uuid: String, database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, &format!("/api/core/snapshots/del/{}", uuid));

    let response = make_http_request(
        "POST",
        &url,
        Some(json!({})),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to delete snapshot: {}", e))
}

#[tauri::command]
pub async fn activate_snapshot(
    uuid: String,
    database: State<'_, Database>,
) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, &format!("/api/core/snapshots/activate/{}", uuid));

    let response = make_http_request(
        "POST",
        &url,
        Some(json!({})),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to activate snapshot: {}", e))
}

#[tauri::command]
pub async fn update_snapshot(
    uuid: String,
    name: String,
    database: State<'_, Database>,
) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, &format!("/api/core/snapshots/set/{}", uuid));

    let payload = json!({
        "uuid": uuid,
        "name": name
    });

    info!("Updating snapshot with payload: {:?}", payload);

    let response = make_http_request(
        "POST",
        &url,
        Some(payload),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to update snapshot: {}", e))
}
