use crate::db::Database;
use crate::http_client::make_http_request;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
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
pub struct NdpDevice {
    mac: String,
    ip: String,
    intf: String,
    manufacturer: String,
    intf_description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NdpResponse {
    total: u32,
    rowCount: u32,
    current: u32,
    rows: Vec<NdpDevice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CombinedDevice {
    mac: String,
    ipv4_addresses: Vec<String>,
    ipv6_addresses: Vec<String>,
    intf: String,
    expired: Option<bool>,
    expires: Option<i32>,
    permanent: Option<bool>,
    device_type: Option<String>,
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

fn is_ipv6(ip: &str) -> bool {
    ip.contains(':')
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
pub async fn get_ndp_devices(database: State<'_, Database>) -> Result<Vec<NdpDevice>, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/diagnostics/interface/search_ndp/");

    let payload = json!({
        "current": 1,
        "rowCount": 1000,
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

    let ndp_response = response
        .json::<NdpResponse>()
        .await
        .map_err(|e| format!("Failed to parse NDP response: {}", e))?;

    Ok(ndp_response.rows)
}

#[tauri::command]
pub async fn get_combined_devices(
    database: State<'_, Database>,
) -> Result<Vec<CombinedDevice>, String> {
    let arp_devices = get_devices(database.clone()).await?;
    let ndp_devices = get_ndp_devices(database.clone()).await?;

    let mut device_map: HashMap<String, CombinedDevice> = HashMap::new();

    for device in arp_devices {
        if let Some(existing_device) = device_map.get_mut(&device.mac) {
            if is_ipv6(&device.ip) {
                if !existing_device.ipv6_addresses.contains(&device.ip) {
                    existing_device.ipv6_addresses.push(device.ip);
                }
            } else {
                if !existing_device.ipv4_addresses.contains(&device.ip) {
                    existing_device.ipv4_addresses.push(device.ip);
                }
            }

            if existing_device.hostname.is_empty() && !device.hostname.is_empty() {
                existing_device.hostname = device.hostname;
            }
        } else {
            let mut ipv4_addresses = Vec::new();
            let mut ipv6_addresses = Vec::new();

            if is_ipv6(&device.ip) {
                ipv6_addresses.push(device.ip);
            } else {
                ipv4_addresses.push(device.ip);
            }

            device_map.insert(
                device.mac.clone(),
                CombinedDevice {
                    mac: device.mac,
                    ipv4_addresses,
                    ipv6_addresses,
                    intf: device.intf,
                    expired: Some(device.expired),
                    expires: Some(device.expires),
                    permanent: Some(device.permanent),
                    device_type: Some(device.device_type),
                    manufacturer: device.manufacturer,
                    hostname: device.hostname,
                    intf_description: device.intf_description,
                },
            );
        }
    }

    for device in ndp_devices {
        if let Some(existing_device) = device_map.get_mut(&device.mac) {
            if is_ipv6(&device.ip) {
                if !existing_device.ipv6_addresses.contains(&device.ip) {
                    existing_device.ipv6_addresses.push(device.ip);
                }
            } else {
                if !existing_device.ipv4_addresses.contains(&device.ip) {
                    existing_device.ipv4_addresses.push(device.ip);
                }
            }

            if existing_device.manufacturer.is_empty() && !device.manufacturer.is_empty() {
                existing_device.manufacturer = device.manufacturer;
            }
        } else {
            let mut ipv4_addresses = Vec::new();
            let mut ipv6_addresses = Vec::new();

            if is_ipv6(&device.ip) {
                ipv6_addresses.push(device.ip);
            } else {
                ipv4_addresses.push(device.ip);
            }

            device_map.insert(
                device.mac.clone(),
                CombinedDevice {
                    mac: device.mac,
                    ipv4_addresses,
                    ipv6_addresses,
                    intf: device.intf,
                    expired: None,
                    expires: None,
                    permanent: None,
                    device_type: None,
                    manufacturer: device.manufacturer,
                    hostname: String::new(),
                    intf_description: device.intf_description,
                },
            );
        }
    }

    let combined_devices: Vec<CombinedDevice> = device_map.into_values().collect();

    Ok(combined_devices)
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
        Some(json!({})),
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
