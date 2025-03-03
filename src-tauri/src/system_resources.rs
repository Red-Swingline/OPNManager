use tauri::State;
use serde::{Serialize, Deserialize};
use crate::db::Database;
use crate::http_client::make_http_request;

#[derive(Serialize, Deserialize, Debug)]
pub struct Memory {
    total: String,
    total_frmt: String,
    used: u64,
    used_frmt: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemResources {
    memory: Memory,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiskDevice {
    device: String,
    #[serde(rename = "type")]
    device_type: String,
    blocks: String,
    used: String,
    available: String,
    used_pct: u8,
    mountpoint: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemDisk {
    devices: Vec<DiskDevice>,
}

#[tauri::command]
pub async fn get_system_resources(database: State<'_, Database>) -> Result<SystemResources, String> {
    let api_info = database.get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!("{}:{}/api/diagnostics/system/systemResources", api_info.api_url, api_info.port);
    
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

    response.json::<SystemResources>().await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn get_system_disk(database: State<'_, Database>) -> Result<SystemDisk, String> {
    let api_info = database.get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!("{}:{}/api/diagnostics/system/systemDisk", api_info.api_url, api_info.port);
    
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

    response.json::<SystemDisk>().await
        .map_err(|e| format!("Failed to parse response: {}", e))
}