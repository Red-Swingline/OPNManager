use crate::db::Database;
use crate::http_client::make_http_request;
use log::{error, info};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InterfaceResponse {
    total: usize,
    rowCount: usize,
    current: usize,
    rows: Vec<Interface>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interface {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    flags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    options: Vec<String>,
    #[serde(default)]
    macaddr: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    supported_media: Vec<String>,
    #[serde(default)]
    is_physical: bool,
    #[serde(default)]
    device: String,
    #[serde(default)]
    mtu: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    macaddr_hw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media_raw: Option<String>,
    #[serde(default)]
    status: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    routes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    groups: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vlan: Option<VlanInfo>,
    #[serde(default)]
    identifier: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    link_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    addr4: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    addr6: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    ipv4: Vec<IpAddress>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    ipv6: Vec<IpAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vlan_tag: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    gateways: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VlanInfo {
    tag: String,
    proto: String,
    pcp: String,
    parent: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IpAddress {
    ipaddr: String,
}

#[tauri::command]
pub async fn get_interfaces(database: State<'_, Database>) -> Result<Vec<Interface>, String> {
    info!("Fetching interface information");
    
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/interfaces/overview/interfacesInfo",
        api_info.api_url, api_info.port
    );

    let mut all_interfaces = Vec::new();
    let mut current_page = 1;
    let mut has_more_pages = true;

    // Fetch all pages
    while has_more_pages {
        info!("Fetching interface page {}", current_page);
        
        let payload = serde_json::json!({
            "current": current_page,
            "rowCount": 10,  // Request more to reduce pagination
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

        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to get response text: {}", e))?;

        // Parse response
        match serde_json::from_str::<InterfaceResponse>(&response_text) {
            Ok(response_data) => {
                // Add interfaces from this page
                all_interfaces.extend(response_data.rows);
                
                // Check if there are more pages
                let total_pages = (response_data.total as f64 / response_data.rowCount as f64).ceil() as usize;
                has_more_pages = current_page < total_pages;
                current_page += 1;
            },
            Err(e) => {
                error!("Failed to parse interface response: {}", e);
                error!("Response: {}", response_text);
                return Err(format!("Failed to parse interface data: {}", e));
            }
        }
    }

    // Sort interfaces by importance - assigned interfaces first, then by device name
    all_interfaces.sort_by(|a, b| {
        // First, sort by whether they have an identifier (assigned interfaces)
        let a_has_id = !a.identifier.is_empty();
        let b_has_id = !b.identifier.is_empty();
        
        match (a_has_id, b_has_id) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                // If both have or don't have IDs, sort by device name
                a.device.cmp(&b.device)
            }
        }
    });

    info!("Retrieved {} interfaces", all_interfaces.len());
    Ok(all_interfaces)
}

#[tauri::command]
pub async fn get_interface_details(device: String, database: State<'_, Database>) -> Result<Interface, String> {
    info!("Getting details for interface: {}", device);
    
    // Get all interfaces and filter for the requested one
    let interfaces = get_interfaces(database).await?;
    
    interfaces
        .into_iter()
        .find(|iface| iface.device == device)
        .ok_or_else(|| format!("Interface '{}' not found", device))
}