use crate::db::Database;
use crate::http_client::make_http_request;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize, Debug)]
pub struct GatewayStatus {
    items: Vec<GatewayItem>,
    status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GatewayItem {
    name: String,
    address: String,
    status: String,
    loss: String,
    delay: String,
    stddev: String,
    status_translated: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServicesResponse {
    total: u32,
    #[serde(rename = "rowCount")]
    row_count: u32,
    current: u32,
    rows: Vec<Service>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Service {
    id: String,
    locked: u8,
    running: u8,
    description: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RestartServiceResponse {
    result: String,
}

#[tauri::command]
pub async fn get_gateway_status(database: State<'_, Database>) -> Result<GatewayStatus, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/routes/gateway/status",
        api_info.api_url, api_info.port
    );

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
        .json::<GatewayStatus>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn get_services(database: State<'_, Database>) -> Result<ServicesResponse, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/core/service/search",
        api_info.api_url, api_info.port
    );

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
        .json::<ServicesResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn restart_service(
    database: State<'_, Database>,
    service_id: String,
) -> Result<RestartServiceResponse, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/core/service/restart/{}",
        api_info.api_url, api_info.port, service_id
    );

    let response = make_http_request(
        "POST",
        &url,
        Some(serde_json::json!({})),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<RestartServiceResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}
