use crate::db::Database;
use crate::http_client::make_http_request;
use log::error;
use reqwest::header::{HeaderMap, ACCEPT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FirewallLog {
    rulenr: Option<String>,
    subrulenr: Option<String>,
    anchorname: Option<String>,
    rid: Option<String>,
    interface: Option<String>,
    reason: Option<String>,
    action: Option<String>,
    dir: Option<String>,
    ipversion: Option<String>,
    tos: Option<String>,
    ecn: Option<String>,
    ttl: Option<String>,
    id: Option<String>,
    offset: Option<String>,
    ipflags: Option<String>,
    protonum: Option<String>,
    protoname: Option<String>,
    length: Option<String>,
    src: Option<String>,
    dst: Option<String>,
    srcport: Option<String>,
    dstport: Option<String>,
    datalen: Option<String>,
    tcpflags: Option<String>,
    seq: Option<String>,
    ack: Option<String>,
    urp: Option<String>,
    tcpopts: Option<String>,
    #[serde(rename = "__timestamp__")]
    timestamp: Option<String>,
    #[serde(rename = "__host__")]
    host: Option<String>,
    #[serde(rename = "__digest__")]
    digest: Option<String>,
    #[serde(rename = "__spec__")]
    spec: Option<Vec<String>>,
    label: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogFilters {
    action: Vec<String>,
    interface_name: Vec<String>,
    dir: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InterfaceNames(pub HashMap<String, String>);

#[tauri::command]
pub async fn get_log_filters(database: State<'_, Database>) -> Result<LogFilters, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/diagnostics/firewall/log_filters",
        api_info.api_url, api_info.port
    );

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".parse().unwrap());

    let response = make_http_request(
        "GET",
        &url,
        None,
        Some(headers),
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<LogFilters>()
        .await
        .map_err(|e| format!("Failed to parse log filters: {}", e))
}

#[tauri::command]
pub async fn get_interface_names(database: State<'_, Database>) -> Result<InterfaceNames, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/diagnostics/interface/getInterfaceNames",
        api_info.api_url, api_info.port
    );

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".parse().unwrap());

    let response = make_http_request(
        "GET",
        &url,
        None,
        Some(headers),
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<InterfaceNames>()
        .await
        .map_err(|e| format!("Failed to parse interface names: {}", e))
}

#[tauri::command]
pub async fn get_firewall_logs(database: State<'_, Database>) -> Result<Vec<FirewallLog>, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/diagnostics/firewall/log",
        api_info.api_url, api_info.port
    );

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".parse().unwrap());

    let response = make_http_request(
        "GET",
        &url,
        None,
        Some(headers),
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to get response text: {}", e))?;

    match serde_json::from_str::<Vec<FirewallLog>>(&response_text) {
        Ok(logs) => Ok(logs),
        Err(e) => {
            error!("Failed to parse logs: {}", e);
            error!(
                "First 1000 chars of response: {}",
                &response_text.chars().take(1000).collect::<String>()
            );
            Err(format!(
                "Failed to parse logs: {}. First 1000 chars of response: {}",
                e,
                &response_text.chars().take(1000).collect::<String>()
            ))
        }
    }
}

#[tauri::command]
pub fn apply_filters(
    logs: Vec<FirewallLog>,
    action: String,
    interface: String,
    direction: String,
) -> Vec<FirewallLog> {
    logs.into_iter()
        .filter(|log| {
            (action.is_empty() || log.action == Some(action.clone()))
                && (interface.is_empty() || log.interface == Some(interface.clone()))
                && (direction.is_empty() || log.dir == Some(direction.clone()))
        })
        .collect()
}

#[tauri::command]
pub fn limit_logs(logs: Vec<FirewallLog>, limit: usize) -> Vec<FirewallLog> {
    logs.into_iter().take(limit).collect()
}
