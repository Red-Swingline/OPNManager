use crate::db::Database;
use crate::http_client::make_http_request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Serialize, Deserialize, Debug)]
pub struct InterfaceTraffic {
    interfaces: HashMap<String, InterfaceData>,
    time: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InterfaceData {
    name: String,
    #[serde(rename = "bytes received")]
    bytes_received: String,
    #[serde(rename = "bytes transmitted")]
    bytes_transmitted: String,
    device: String,
    driver: String,
}

#[tauri::command]
pub async fn get_interface_traffic(
    database: State<'_, Database>,
) -> Result<InterfaceTraffic, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/diagnostics/traffic/interface",
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
        .json::<InterfaceTraffic>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}
