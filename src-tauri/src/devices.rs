use crate::db::Database;
use crate::http_client::make_http_request;
use log::{info, warn};
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
    // Start time tracking for performance monitoring
    let start_time = std::time::Instant::now();

    // Create an async function to fetch devices with timeout
    async fn fetch_with_timeout<T, E>(
        fut: impl std::future::Future<Output = Result<T, E>>,
        timeout_seconds: u64,
        default: T,
        data_type: &str,
    ) -> Result<T, E> {
        let fetch_start = std::time::Instant::now();

        match tokio::time::timeout(std::time::Duration::from_secs(timeout_seconds), fut).await {
            Ok(result) => {
                // Calculate fetch duration
                let fetch_duration = fetch_start.elapsed();
                info!(
                    "{} data fetch completed in {}ms",
                    data_type,
                    fetch_duration.as_millis()
                );
                result
            }
            Err(_) => {
                let error_msg = format!(
                    "{} fetch timed out after {} seconds",
                    data_type, timeout_seconds
                );
                warn!("{}", error_msg);
                Ok(default)
            }
        }
    }

    // Set timeouts
    let main_timeout = 30; // 30 seconds for main fetch

    // First, try to fetch both in parallel with the main timeout
    let arp_future = get_devices(database.clone());
    let ndp_future = get_ndp_devices(database.clone());

    // Start the futures in parallel
    let (arp_result, ndp_result) = tokio::join!(
        fetch_with_timeout(arp_future, main_timeout, Vec::new(), "ARP"),
        fetch_with_timeout(ndp_future, main_timeout, Vec::new(), "NDP")
    );

    // Check for errors and extract results
    let arp_devices = arp_result?;
    let ndp_devices = ndp_result?;

    // Store the counts for later use
    let arp_count = arp_devices.len();
    let ndp_count = ndp_devices.len();

    info!(
        "Combining device data: {} ARP devices and {} NDP devices",
        arp_count, ndp_count
    );

    // Pre-allocate hash map with capacity to avoid reallocations
    let mut device_map: HashMap<String, CombinedDevice> =
        HashMap::with_capacity(arp_devices.len() + ndp_devices.len());

    // Process ARP devices first
    for device in arp_devices {
        if let Some(existing_device) = device_map.get_mut(&device.mac) {
            // Update existing device
            if is_ipv6(&device.ip) {
                if !existing_device.ipv6_addresses.contains(&device.ip) {
                    existing_device.ipv6_addresses.push(device.ip);
                }
            } else if !existing_device.ipv4_addresses.contains(&device.ip) {
                existing_device.ipv4_addresses.push(device.ip);
            }

            if existing_device.hostname.is_empty() && !device.hostname.is_empty() {
                existing_device.hostname = device.hostname;
            }
        } else {
            // Create new device entry
            let mut ipv4_addresses = Vec::with_capacity(1);
            let mut ipv6_addresses = Vec::with_capacity(1);

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

    // Process NDP devices
    for device in ndp_devices {
        if let Some(existing_device) = device_map.get_mut(&device.mac) {
            if is_ipv6(&device.ip) {
                if !existing_device.ipv6_addresses.contains(&device.ip) {
                    existing_device.ipv6_addresses.push(device.ip);
                }
            } else if !existing_device.ipv4_addresses.contains(&device.ip) {
                existing_device.ipv4_addresses.push(device.ip);
            }

            if existing_device.manufacturer.is_empty() && !device.manufacturer.is_empty() {
                existing_device.manufacturer = device.manufacturer;
            }
        } else {
            let mut ipv4_addresses = Vec::with_capacity(1);
            let mut ipv6_addresses = Vec::with_capacity(1);

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

    // Sort device addresses
    for device in device_map.values_mut() {
        if device.ipv4_addresses.len() > 1 {
            device.ipv4_addresses.sort_by(|a, b| natural_sort(a, b));
        }
        if device.ipv6_addresses.len() > 1 {
            device.ipv6_addresses.sort();
        }
    }

    // Filter for devices with CARP interfaces in HA setups
    // Look for devices connected to CARP interfaces and normalize their data
    let device_map = device_map
        .into_iter()
        .map(|(mac, mut device)| {
            // Check if device is on a CARP interface
            let is_on_carp_iface = device.intf.contains("_vip")
                || device.intf.starts_with("carp")
                || device.intf_description.to_lowercase().contains("carp");

            // If it's on a CARP interface, tag it for special handling
            if is_on_carp_iface {
                device.intf_description = format!("{} (CARP VIP)", device.intf_description);
            }

            (mac, device)
        })
        .collect::<HashMap<String, CombinedDevice>>();

    // Convert to vector and sort
    let mut combined_devices: Vec<CombinedDevice> = device_map.into_values().collect();

    info!("Total combined devices: {}", combined_devices.len());

    // Sort devices by interface first, then by IP address
    combined_devices.sort_by(|a, b| {
        let intf_cmp = a.intf.cmp(&b.intf);
        if intf_cmp != std::cmp::Ordering::Equal {
            return intf_cmp;
        }

        // IPv4 addresses take precedence
        let a_has_ipv4 = !a.ipv4_addresses.is_empty();
        let b_has_ipv4 = !b.ipv4_addresses.is_empty();

        match (a_has_ipv4, b_has_ipv4) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (true, true) => natural_sort(&a.ipv4_addresses[0], &b.ipv4_addresses[0]),
            (false, false) => {
                // Compare IPv6 addresses if no IPv4 addresses
                if !a.ipv6_addresses.is_empty() && !b.ipv6_addresses.is_empty() {
                    a.ipv6_addresses[0].cmp(&b.ipv6_addresses[0])
                } else if a.ipv6_addresses.is_empty() && !b.ipv6_addresses.is_empty() {
                    std::cmp::Ordering::Greater
                } else if !a.ipv6_addresses.is_empty() && b.ipv6_addresses.is_empty() {
                    std::cmp::Ordering::Less
                } else {
                    // Last resort: compare by MAC address
                    a.mac.cmp(&b.mac)
                }
            }
        }
    });

    // Calculate and log total operation duration
    let total_duration = start_time.elapsed();
    info!(
        "Device processing completed in {}ms: {} total devices processed",
        total_duration.as_millis(),
        combined_devices.len()
    );

    // Log performance issues
    if total_duration.as_secs() > 5 || combined_devices.len() > 100 {
        // This is a complex network that might benefit from performance mode
        warn!(
            "Complex network detected: {} devices processed in {}ms",
            combined_devices.len(),
            total_duration.as_millis()
        );
    }

    Ok(combined_devices)
}

fn natural_sort(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<&str> = a.split('.').collect();
    let b_parts: Vec<&str> = b.split('.').collect();

    for i in 0..4 {
        if i >= a_parts.len() || i >= b_parts.len() {
            return a_parts.len().cmp(&b_parts.len());
        }

        let a_num = a_parts[i].parse::<u32>().unwrap_or(0);
        let b_num = b_parts[i].parse::<u32>().unwrap_or(0);

        match a_num.cmp(&b_num) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    std::cmp::Ordering::Equal
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
