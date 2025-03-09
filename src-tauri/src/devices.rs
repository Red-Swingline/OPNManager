use crate::db::Database;
use crate::http_client::make_http_request;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
    mac: String,
    ip: String,
    intf: String,
    expired: bool,
    expires: i32,
    permanent: bool,
    #[serde(rename = "type")]
    device_type: String,
    manufacturer: String,
    hostname: String,
    intf_description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlushArpResponse {
    deleted: Vec<String>,
}

fn build_api_url(api_info: &crate::db::ApiInfo, endpoint: &str) -> String {
    format!("{}:{}{}", api_info.api_url, api_info.port, endpoint)
}

#[tauri::command]
pub async fn get_devices(database: State<'_, Database>) -> Result<Vec<Device>, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/diagnostics/interface/getArp");

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
        .json::<Vec<Device>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn flush_arp_table(database: State<'_, Database>) -> Result<FlushArpResponse, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/diagnostics/interface/flushArp");

    let response = make_http_request(
        "POST",
        &url,
        Some(json!({})), // Empty JSON object as payload
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to get response body: {}", e))?;

    let deleted: Vec<String> = body
        .lines()
        .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
        .filter(|ip| !ip.is_empty())
        .collect();

    Ok(FlushArpResponse { deleted })
}
