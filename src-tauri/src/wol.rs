use crate::db::Database;
use crate::http_client::make_http_request;
use serde_json::{json, Value};
use tauri::State;

// Check if WoL plugin is installed by checking if UI page exists
#[tauri::command]
pub async fn check_wol_plugin_installed(database: State<'_, Database>) -> Result<bool, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    // Try to access the WoL API endpoint
    let url = format!("{}:{}/api/wol/wol/getwake", api_info.api_url, api_info.port);

    match make_http_request(
        "GET",
        &url,
        None,
        None,
        Some(10), // Short timeout
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await
    {
        Ok(_) => Ok(true),
        Err(_) => Ok(false), // If error, assume plugin is not installed
    }
}

// Get available interfaces for WoL
#[tauri::command]
pub async fn get_wol_interfaces(database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!("{}:{}/api/wol/wol/getwake", api_info.api_url, api_info.port);

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

    // Return just the interface list
    if let Some(wake) = result.get("wake") {
        if let Some(interfaces) = wake.get("interface") {
            return Ok(interfaces.clone());
        }
    }

    Err("Interface list not found".to_string())
}

// Get configured WoL hosts
#[tauri::command]
pub async fn search_wol_hosts(database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/wol/wol/searchHost",
        api_info.api_url, api_info.port
    );

    let payload = json!({
        "current": 1,
        "rowCount": -1,
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
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

// Get ARP table devices for dropdown selection
#[tauri::command]
pub async fn get_arp_devices(database: State<'_, Database>) -> Result<Value, String> {
    // We'll use the existing devices API endpoint to get the ARP table
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/diagnostics/interface/getArp",
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
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

// Wake a device by UUID from saved devices
#[tauri::command]
pub async fn wake_device(database: State<'_, Database>, uuid: String) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!("{}:{}/api/wol/wol/set", api_info.api_url, api_info.port);

    // The OPNsense WoL API expects the UUID in a specific format
    let payload = json!({
        "wake": {
            "uuid": uuid
        }
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
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

// Send WoL to a MAC address directly (from dropdown selection)
#[tauri::command]
pub async fn wake_mac_address(
    database: State<'_, Database>,
    interface: String,
    mac: String,
    description: String,
) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!("{}:{}/api/wol/wol/set", api_info.api_url, api_info.port);

    // Create a temporary configuration for this wake request
    let payload = json!({
        "wake": {
            "interface": interface,
            "mac": mac,
            "descr": description
        }
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
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

// Add a new WoL host
#[tauri::command]
pub async fn add_wol_host(
    database: State<'_, Database>,
    interface: String,
    mac: String,
    description: String,
) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/wol/wol/addHost/",
        api_info.api_url, api_info.port
    );

    let payload = json!({
        "host": {
            "interface": interface,
            "mac": mac,
            "descr": description
        }
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
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

// Delete a WoL host
#[tauri::command]
pub async fn delete_wol_host(database: State<'_, Database>, uuid: String) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/wol/wol/delHost/{}",
        api_info.api_url, api_info.port, uuid
    );

    // Make the request, but we don't need the response body
    let _response = make_http_request(
        "POST",
        &url,
        Some(json!({})),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    // The delete endpoint returns an empty response when successful
    Ok(json!({"status": "OK"}))
}

// Start installation of WoL plugin
#[tauri::command]
pub async fn install_wol_plugin(database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    // The correct endpoint includes the package name in the URL
    let url = format!(
        "{}:{}/api/core/firmware/install/os-wol",
        api_info.api_url, api_info.port
    );

    // Send an empty JSON object as the payload
    let payload = json!({});

    let response = make_http_request(
        "POST",
        &url,
        Some(payload),
        None,
        Some(10), // Short timeout for just starting the installation
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let result = response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Return the message UUID for status checking
    Ok(result)
}

// Check the status of a plugin installation
#[tauri::command]
pub async fn check_install_status(database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    // Add a timestamp parameter to prevent caching
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let url = format!(
        "{}:{}/api/core/firmware/upgradestatus?v={}",
        api_info.api_url, api_info.port, timestamp
    );

    let response = make_http_request(
        "GET",
        &url,
        None,
        None,
        Some(10),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}
