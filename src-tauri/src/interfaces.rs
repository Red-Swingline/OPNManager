use crate::db::Database;
use crate::http_client::make_http_request;
use log::{error, info, warn};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InterfaceResponse {
    #[serde(default)]
    #[serde(default)]
    total: usize,
    #[serde(default)]
    #[serde(default)]
    rowCount: usize,
    #[serde(default)]
    #[serde(default)]
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

    // Track the start time for performance measurements
    let start_time = std::time::Instant::now();

    // Track the start time for performance measurements
    let start_time = std::time::Instant::now();

    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/interfaces/overview/interfacesInfo",
        api_info.api_url, api_info.port
    );

    // Increase page size to get more interfaces in fewer requests
    let page_size = 50; // Increased from 25
    // Increase page size to get more interfaces in fewer requests
    let page_size = 50; // Increased from 25
    let mut all_interfaces = Vec::new();
    let mut current_page = 1;

    // Implement retry logic with backoff
    let max_retries = 3;
    let initial_timeout = 15; // 15 seconds initial timeout per request

    // Implement retry logic with backoff
    let max_retries = 3;
    let initial_timeout = 15; // 15 seconds initial timeout per request

    // Set a timeout for the entire operation
    let timeout = tokio::time::timeout(
        std::time::Duration::from_secs(45), // 45 second global timeout (increased from 30)
        async {
            loop {
                info!("Fetching interface page {}", current_page);
                let payload = serde_json::json!({
                    "current": current_page,
                    "rowCount": page_size,
                    "sort": {},
                    "searchPhrase": ""
                });
    // Set a timeout for the entire operation
    let timeout = tokio::time::timeout(
        std::time::Duration::from_secs(45), // 45 second global timeout (increased from 30)
        async {
            loop {
                info!("Fetching interface page {}", current_page);
                let payload = serde_json::json!({
                    "current": current_page,
                    "rowCount": page_size,
                    "sort": {},
                    "searchPhrase": ""
                });

                // Try with retries and backoff
                let mut retry_count = 0;
                let mut last_error = String::new();
                let mut current_timeout = initial_timeout;

                while retry_count < max_retries {
                    match make_http_request(
                        "POST",
                        &url,
                        Some(payload.clone()),
                        None,
                        Some(current_timeout),
                        Some(&api_info.api_key),
                        Some(&api_info.api_secret),
                    ).await {
                        Ok(response) => {
                            match response.text().await {
                                Ok(response_text) => {
                                    // Parse response
                                    match serde_json::from_str::<InterfaceResponse>(&response_text) {
                                        Ok(response_data) => {
                                            // Add interfaces from this page
                                            all_interfaces.extend(response_data.rows.clone());
                // Try with retries and backoff
                let mut retry_count = 0;
                let mut last_error = String::new();
                let mut current_timeout = initial_timeout;

                while retry_count < max_retries {
                    match make_http_request(
                        "POST",
                        &url,
                        Some(payload.clone()),
                        None,
                        Some(current_timeout),
                        Some(&api_info.api_key),
                        Some(&api_info.api_secret),
                    ).await {
                        Ok(response) => {
                            match response.text().await {
                                Ok(response_text) => {
                                    // Parse response
                                    match serde_json::from_str::<InterfaceResponse>(&response_text) {
                                        Ok(response_data) => {
                                            // Add interfaces from this page
                                            all_interfaces.extend(response_data.rows.clone());

                                            // Check if there are more pages
                                            let total_pages =
                                                (response_data.total as f64 / response_data.rowCount as f64).ceil() as usize;

                                            // Log progress
                                            info!(
                                                "Received {}/{} interfaces (page {}/{})",
                                                all_interfaces.len(),
                                                response_data.total,
                                                current_page,
                                                total_pages
                                            );

                                            // Check if we're done
                                            if current_page >= total_pages {
                                                return Ok(());
                                            }

                                            // Move to next page
                                            current_page += 1;
                                            break; // Break out of retry loop on success
                                        },
                                        Err(e) => {
                                            error!("Failed to parse interface response: {}", e);

                                            // Try to log part of the response text for debugging (truncated if very large)
                                            let preview_length = std::cmp::min(500, response_text.len());
                                            let preview = &response_text[..preview_length];
                                            error!("Response text preview: {}{}", 
                                                  preview,
                                                  if preview_length < response_text.len() { "..." } else { "" });

                                            // Try to identify specific parsing issues
                                            if e.to_string().contains("missing field") {
                                                let error_str = e.to_string();
                                                let field = error_str
                                                    .split("missing field `")
                                                    .nth(1)
                                                    .and_then(|s| s.split('`').next())
                                                    .unwrap_or("unknown");

                                                error!("JSON missing required field: {}", field);
                                                last_error = format!("Response missing required field '{}'. This might indicate an interface with unusual configuration.", field);
                                            } else if e.to_string().contains("expected") && e.to_string().contains("found") {
                                                // Type mismatch error
                                                error!("JSON type mismatch: {}", e);
                                                last_error = format!("Type mismatch in response: {}. This could be caused by an interface with unexpected properties.", e);
                                            } else {
                                                // Generic parsing error
                                                last_error = format!("Failed to parse interface data: {}", e);
                                            }

                                            // Attempt to salvage data by parsing as generic JSON
                                            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
                                                // Check if it's an actual API error message
                                                if let Some(message) = json_value.get("message").and_then(|m| m.as_str()) {
                                                    error!("API returned an error message: {}", message);
                                                    last_error = format!("API error: {}", message);
                                                } else if let Some(rows) = json_value.get("rows") {
                                                    // Try to extract interfaces even with parsing errors
                                                    if let Some(rows_array) = rows.as_array() {
                                                        info!("Found {} interfaces in rows array despite parsing errors", rows_array.len());

                                                        // Try to log some details about the problematic interfaces
                                                        for (i, iface) in rows_array.iter().enumerate().take(5) {
                                                            if let Some(device) = iface.get("device").and_then(|d| d.as_str()) {
                                                                info!("Interface {}: {} (logging first few to identify issues)", i, device);
                                                            }
                                                        }

                                                        // Add additional context to error message
                                                        last_error = format!("{}. Found {} interfaces but couldn't parse them properly. This may be due to unusual interface properties.", 
                                                                           last_error, rows_array.len());
                                                    }
                                                }
                                            }

                                            // Continue to retry, but with a different approach each time
                                            retry_count += 1;
                                            current_timeout += 5; // Increase timeout by 5 seconds for each retry

                                            // Log detailed retry information
                                            warn!("Retry {}/{} for interface page {} after parse error: {}", 
                                                  retry_count, max_retries, current_page, last_error);

                                            // Sleep before retry with increasing backoff
                                            tokio::time::sleep(std::time::Duration::from_millis(500 * retry_count as u64)).await;
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!("Failed to get response text: {}", e);
                                    last_error = format!("Failed to read response: {}", e);
                                    retry_count += 1;
                                    current_timeout += 5;
                                    warn!("Retry {}/{} for interface page {} after text error", 
                                          retry_count, max_retries, current_page);
                                    tokio::time::sleep(std::time::Duration::from_millis(500 * retry_count as u64)).await;
                                }
                            }
                        },
                        Err(e) => {
                            error!("Request error for interface page {}: {}", current_page, e);
                            last_error = e;
                            retry_count += 1;
                            current_timeout += 5;

                            // Special handling for common errors
                            if last_error.contains("timeout") || last_error.contains("timed out") {
                                warn!("Timeout detected, increasing timeout for retry");
                                current_timeout += 10; // Add extra time for timeout errors
                            }

                            warn!("Retry {}/{} for interface page {} after request error", 
                                  retry_count, max_retries, current_page);
                            tokio::time::sleep(std::time::Duration::from_millis(500 * retry_count as u64)).await;
                        }
                    }
                }

                // If we've gone through all retries and still failed
                if retry_count >= max_retries {
                    if all_interfaces.is_empty() {
                        // No interfaces retrieved yet - try alternative approach
                        info!("Main interface fetch failed after retries. Trying alternative approach...");
                        // Will fall through to the fallback mechanism outside the timeout
                        return Err(format!("Failed to fetch interfaces after {} retries: {}", max_retries, last_error));
                    } else {
                        // We have some interfaces - return what we have with a warning
                        warn!("Partial interface data fetched ({} interfaces). Some interfaces may be missing.", 
                               all_interfaces.len());
                        return Ok(());
                    }
                }
            }
        }
    ).await;

    // Handle timeout or errors
    if let Err(_) = timeout {
        error!("Interface fetch timed out after 45 seconds");
        if all_interfaces.is_empty() {
            // If no interfaces were retrieved, try our fallback approach
            info!("Attempting fallback interface retrieval method...");
            return try_alternative_interface_fetch(&api_info).await
                .map_err(|e| format!("All interface fetch methods failed. Primary: timeout after 45s. Fallback: {}", e));
        } else {
            // Return what we have so far with a warning
            info!(
                "Returning {} interfaces collected before timeout",
                all_interfaces.len()
            );
        }
    } else if let Err(e) = timeout.unwrap() {
        // Internal error occurred and we got no interfaces, try fallback
        if all_interfaces.is_empty() {
            info!(
                "Primary interface fetch failed: {}. Trying fallback method...",
                e
            );
            return try_alternative_interface_fetch(&api_info)
                .await
                .map_err(|e2| {
                    format!(
                        "All interface fetch methods failed. Primary: {}. Fallback: {}",
                        e, e2
                    )
                });
        } else {
            // We have partial data, log the error but return what we have
            warn!(
                "Interface fetch encountered errors but retrieved {} interfaces: {}",
                all_interfaces.len(),
                e
            );
        }
    }

    // Process interfaces - handle HA and CARP interfaces specially
    // Filter out duplicate or redundant interfaces from HA configurations
    let mut seen_devices = std::collections::HashSet::new();
    let mut filtered_interfaces = Vec::with_capacity(all_interfaces.len());

    for iface in &all_interfaces {
        // Check for CARP interfaces or other HA indicators
        let is_carp = iface.device.contains("_vip")
            || iface.device.starts_with("carp")
            || (iface.description.to_lowercase().contains("carp")
                && iface.description.to_lowercase().contains("vip"))
            || (iface
                .groups
                .iter()
                .any(|g| g.to_lowercase().contains("carp")));

        // For non-CARP interfaces or unique CARP interfaces, keep them
        if !is_carp || !seen_devices.contains(&iface.device) {
            seen_devices.insert(iface.device.clone());
            filtered_interfaces.push(iface.clone());
        } else {
            // Log that we're filtering out a duplicate CARP interface
            info!("Filtering out duplicate CARP interface: {}", iface.device);
        }
    }

    // More comprehensive detection of HA setups
    let carp_count = all_interfaces
        .iter()
        .filter(|iface| {
            iface.device.contains("_vip")
                || iface.device.starts_with("carp")
                || (iface.description.to_lowercase().contains("carp")
                    && iface.description.to_lowercase().contains("vip"))
                || (iface
                    .groups
                    .iter()
                    .any(|g| g.to_lowercase().contains("carp")))
        })
        .count();

    // Calculate load duration
    let duration = start_time.elapsed();
    let duration_ms = duration.as_millis() as u64;

    // Log the HA indicators
    info!("HA indicators - carp_interfaces: {}, filtered_out: {}, total_interfaces: {}, duration_ms: {}",
        carp_count,
        all_interfaces.len() - filtered_interfaces.len(),
        all_interfaces.len(),
        duration_ms
    );

    // Detect HA setup based on multiple indicators
    let is_ha_setup = carp_count > 0
        || (all_interfaces.len() - filtered_interfaces.len() > all_interfaces.len() / 10);

    if is_ha_setup {
        let ha_info = format!(
            "Detected HA setup - found {} CARP interfaces, filtered out {} of {} interfaces in {}ms",
            carp_count,
            all_interfaces.len() - filtered_interfaces.len(),
            all_interfaces.len(),
            duration.as_millis()
        );
        info!("{}", ha_info);

        // Log detailed information about the HA setup
        // Since we can't easily get the AppHandle in this context, we'll log without using diagnostics
        {
            let mut data = std::collections::HashMap::new();
            data.insert(
                "total_interfaces".to_string(),
                all_interfaces.len().to_string(),
            );
            data.insert(
                "filtered_interfaces".to_string(),
                filtered_interfaces.len().to_string(),
            );
            data.insert("carp_interfaces".to_string(), carp_count.to_string());
            data.insert(
                "filtered_out".to_string(),
                (all_interfaces.len() - filtered_interfaces.len()).to_string(),
            );
            data.insert("duration_ms".to_string(), duration_ms.to_string());

            // Log virtual IP assignments if present
            let vips: Vec<_> = all_interfaces
                .iter()
                .filter(|iface| iface.device.contains("_vip") || iface.device.starts_with("carp"))
                .map(|iface| format!("{}:{}", iface.device, iface.status))
                .collect();

            if !vips.is_empty() {
                data.insert("virtual_ips".to_string(), vips.join(","));
            }

            // Instead of using diagnostics, log to console for now
            info!("HA diagnostics data: {:?}", data);
        }
    }

    // Replace original with filtered
    all_interfaces = filtered_interfaces;

    // Sort interfaces - prioritize active and assigned interfaces
    all_interfaces.sort_by(|a, b| {
        // First, sort by status (up > down)
        let a_is_up = a.status.to_lowercase() == "up";
        let b_is_up = b.status.to_lowercase() == "up";

        match (a_is_up, b_is_up) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        // Special handling for CARP - prioritize physical interfaces over CARP in HA setups
        let a_is_carp = a.device.contains("_vip") || a.device.starts_with("carp");
        let b_is_carp = b.device.contains("_vip") || b.device.starts_with("carp");

        match (a_is_carp, b_is_carp) {
            (false, true) => return std::cmp::Ordering::Less, // Physical interfaces before CARP
            (true, false) => return std::cmp::Ordering::Greater,
            _ => {} // Both physical or both CARP, continue with regular sorting
        }

        // Then sort by whether they have an identifier (assigned interfaces)
        // First, sort by status (up > down)
        let a_is_up = a.status.to_lowercase() == "up";
        let b_is_up = b.status.to_lowercase() == "up";

        match (a_is_up, b_is_up) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        // Special handling for CARP - prioritize physical interfaces over CARP in HA setups
        let a_is_carp = a.device.contains("_vip") || a.device.starts_with("carp");
        let b_is_carp = b.device.contains("_vip") || b.device.starts_with("carp");

        match (a_is_carp, b_is_carp) {
            (false, true) => return std::cmp::Ordering::Less, // Physical interfaces before CARP
            (true, false) => return std::cmp::Ordering::Greater,
            _ => {} // Both physical or both CARP, continue with regular sorting
        }

        // Then sort by whether they have an identifier (assigned interfaces)
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

    // Calculate load duration
    let duration = start_time.elapsed();
    let duration_ms = duration.as_millis() as u64;

    info!(
        "Retrieved {} interfaces in {}ms",
        all_interfaces.len(),
        duration_ms
    );

    // Log performance metrics to console instead of diagnostics system
    let performance_data = format!(
        "Interface loading stats: {} interfaces loaded in {}ms",
        all_interfaces.len(),
        duration_ms
    );

    if duration_ms > 5000 {
        // Log as a potential performance issue if taking over 5 seconds
        warn!("PERFORMANCE ISSUE: {}", performance_data);
    } else {
        info!("{}", performance_data);
    }

    // Calculate load duration
    let duration = start_time.elapsed();
    let duration_ms = duration.as_millis() as u64;

    info!(
        "Retrieved {} interfaces in {}ms",
        all_interfaces.len(),
        duration_ms
    );

    // Log performance metrics to console instead of diagnostics system
    let performance_data = format!(
        "Interface loading stats: {} interfaces loaded in {}ms",
        all_interfaces.len(),
        duration_ms
    );

    if duration_ms > 5000 {
        // Log as a potential performance issue if taking over 5 seconds
        warn!("PERFORMANCE ISSUE: {}", performance_data);
    } else {
        info!("{}", performance_data);
    }

    Ok(all_interfaces)
}

// Alternative interface fetch method that tries a different API endpoint
async fn try_alternative_interface_fetch(
    api_info: &crate::db::ApiInfo,
) -> Result<Vec<Interface>, String> {
    info!("Using alternative interface fetch method");

    // Try multiple alternative endpoints (in order of preference)
    let endpoints = [
        "/api/diagnostics/interface/getInterfaces", // First choice - most reliable
        "/api/diagnostics/netstat/interface",       // Second choice - alternative format
        "/api/interfaces/search_interfaces",        // Third choice - different layout
    ];

    // Track the endpoints we've tried and their errors
    let mut endpoint_errors = Vec::new();

    // Try each endpoint until one works
    for endpoint in &endpoints {
        let url = format!("{}:{}{}", api_info.api_url, api_info.port, endpoint);
        info!("Trying alternative endpoint: {}", endpoint);

        match fetch_interfaces_from_endpoint(&url, api_info).await {
            Ok(interfaces) => {
                info!(
                    "Successfully retrieved {} interfaces from endpoint {}",
                    interfaces.len(),
                    endpoint
                );
                return Ok(interfaces);
            }
            Err(e) => {
                warn!("Endpoint {} failed: {}", endpoint, e);
                endpoint_errors.push(format!("{}: {}", endpoint, e));
                // Continue to the next endpoint
            }
        }
    }

    // Finally, try one more desperate approach - get any kind of interface data
    info!("Trying desperate measure to get any interface data");
    match try_extract_any_interfaces(api_info).await {
        Ok(interfaces) => {
            info!(
                "Desperate measure succeeded in getting {} interfaces",
                interfaces.len()
            );
            return Ok(interfaces);
        }
        Err(e) => {
            error!("All interface fetching methods failed");
            return Err(format!(
                "All alternative methods failed. Errors: {}. Last error: {}",
                endpoint_errors.join("; "),
                e
            ));
        }
    }
}

// Helper to fetch interfaces from a specific endpoint
async fn fetch_interfaces_from_endpoint(
    url: &str,
    api_info: &crate::db::ApiInfo,
) -> Result<Vec<Interface>, String> {
    // Try both GET and POST methods
    let methods = ["GET", "POST"];
    let mut last_error = String::new();

    for method in &methods {
        info!("Trying {} request to {}", method, url);

        let payload = if *method == "POST" {
            Some(serde_json::json!({
                "current": 1,
                "rowCount": 100,  // Request a large number of interfaces at once
                "searchPhrase": ""
            }))
        } else {
            None
        };

        match make_http_request(
            method,
            url,
            payload,
            None,
            Some(20), // 20 second timeout
            Some(&api_info.api_key),
            Some(&api_info.api_secret),
        )
        .await
        {
            Ok(response) => {
                match response.text().await {
                    Ok(response_text) => {
                        info!("Received response from {}", url);

                        // Try to determine the response format and parse accordingly
                        if let Some(interfaces) = try_parse_interface_response(&response_text) {
                            return Ok(interfaces);
                        } else {
                            last_error = format!("Could not parse interface data from {}", url);
                        }
                    }
                    Err(e) => {
                        last_error = format!("Failed to read response from {}: {}", url, e);
                    }
                }
            }
            Err(e) => {
                last_error = format!("{} request to {} failed: {}", method, url, e);
                // Continue to the next method
            }
        }
    }

    Err(last_error)
}

// Parse interface response based on its format
fn try_parse_interface_response(response_text: &str) -> Option<Vec<Interface>> {
    // Try different parsing strategies

    // Try parsing as InterfaceResponse first (primary format)
    if let Ok(response_data) = serde_json::from_str::<InterfaceResponse>(response_text) {
        return Some(response_data.rows);
    }

    // Try parsing as direct array of interfaces
    if let Ok(interfaces) = serde_json::from_str::<Vec<Interface>>(response_text) {
        return Some(interfaces);
    }

    // Try parsing as array of generic JSON values (fallback format)
    if let Ok(json_array) = serde_json::from_str::<Vec<serde_json::Value>>(response_text) {
        // Convert generic JSON to Interface structs
        let interfaces = convert_json_to_interfaces(&json_array);
        if !interfaces.is_empty() {
            return Some(interfaces);
        }
    }

    // Try parsing as object with rows array (another common format)
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response_text) {
        if let Some(rows) = json_value.get("rows").and_then(|r| r.as_array()) {
            let interfaces = convert_json_to_interfaces(rows);
            if !interfaces.is_empty() {
                return Some(interfaces);
            }
        }
    }

    None
}

// Convert JSON values to Interface structs
fn convert_json_to_interfaces(json_array: &[serde_json::Value]) -> Vec<Interface> {
    let mut interfaces = Vec::new();

    for item in json_array {
        // First try to get the device name from various possible fields
        let device_name = item
            .get("device")
            .or_else(|| item.get("if"))
            .or_else(|| item.get("interface"))
            .or_else(|| item.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if device_name.is_empty() {
            continue; // Skip items without a device name
        }

        // Get other fields with flexible fallbacks
        let status = get_string_value(item, &["status", "state", "up"])
            .unwrap_or_else(|| "unknown".to_string());
        let macaddr = get_string_value(item, &["macaddr", "mac", "ether"]).unwrap_or_default();
        let mtu = get_string_value(item, &["mtu"]).unwrap_or_default();
        let media = get_string_value(item, &["media", "type"]);
        let description = get_string_value(item, &["description", "descr", "desc", "comment"])
            .unwrap_or_default();

        // Clone status for enabled check
        let status_lowercase = status.to_lowercase();

        // Build interface with available information
        let interface = Interface {
            device: device_name.to_string(),
            status,
            macaddr,
            mtu,
            media,
            description,
            is_physical: !device_name.contains(".") && !device_name.contains(":"),
            // Set minimal defaults for other fields
            flags: Vec::new(),
            capabilities: Vec::new(),
            options: Vec::new(),
            supported_media: Vec::new(),
            macaddr_hw: None,
            media_raw: None,
            routes: Vec::new(),
            config: None,
            groups: Vec::new(),
            vlan: None,
            identifier: "".to_string(),
            enabled: status_lowercase == "up",
            link_type: None,
            addr4: None,
            addr6: None,
            ipv4: Vec::new(),
            ipv6: Vec::new(),
            vlan_tag: None,
            gateways: Vec::new(),
        };

        interfaces.push(interface);
    }

    interfaces
}

// Helper to extract string values from JSON with multiple possible field names
fn get_string_value(json: &serde_json::Value, possible_fields: &[&str]) -> Option<String> {
    for field in possible_fields {
        if let Some(value) = json.get(*field) {
            if let Some(s) = value.as_str() {
                return Some(s.to_string());
            } else if let Some(n) = value.as_u64() {
                return Some(n.to_string());
            } else if value.is_boolean() {
                return Some(value.as_bool().unwrap().to_string());
            }
        }
    }
    None
}

// Last-resort attempt to get any interface data
async fn try_extract_any_interfaces(
    api_info: &crate::db::ApiInfo,
) -> Result<Vec<Interface>, String> {
    // Try to get status page data which usually contains interface information
    let url = format!(
        "{}:{}/api/core/system/status",
        api_info.api_url, api_info.port
    );

    match make_http_request(
        "GET",
        &url,
        None,
        None,
        Some(15),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await
    {
        Ok(response) => {
            if let Ok(text) = response.text().await {
                // Try to extract any interface-looking data
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    // Search for arrays that might contain interface data
                    if let Some(interfaces) = extract_interfaces_from_json(&json) {
                        if !interfaces.is_empty() {
                            return Ok(interfaces);
                        }
                    }
                }
            }
        }
        Err(_) => {}
    }

    // If all else fails, we could create a dummy interface, but let's just return an error
    // for better transparency to the user

    Err("Unable to retrieve any interface data from the firewall. The API may not be compatible or accessible.".to_string())
}

// Recursively search JSON for interface data
fn extract_interfaces_from_json(json: &serde_json::Value) -> Option<Vec<Interface>> {
    match json {
        serde_json::Value::Array(arr) => {
            // If this array looks like interfaces, convert it
            if arr.len() > 0
                && arr.iter().any(|item| {
                    item.get("if").is_some()
                        || item.get("interface").is_some()
                        || item.get("device").is_some()
                })
            {
                Some(convert_json_to_interfaces(arr))
            } else {
                // Search each array element
                for item in arr {
                    if let Some(interfaces) = extract_interfaces_from_json(item) {
                        if !interfaces.is_empty() {
                            return Some(interfaces);
                        }
                    }
                }
                None
            }
        }
        serde_json::Value::Object(obj) => {
            // Check if this object has interfaces or network keys
            for key in &["interfaces", "network", "interface", "ifconfig"] {
                if let Some(value) = obj.get(*key) {
                    if let Some(interfaces) = extract_interfaces_from_json(value) {
                        if !interfaces.is_empty() {
                            return Some(interfaces);
                        }
                    }
                }
            }

            // Search through all object values
            for value in obj.values() {
                if let Some(interfaces) = extract_interfaces_from_json(value) {
                    if !interfaces.is_empty() {
                        return Some(interfaces);
                    }
                }
            }

            None
        }
        _ => None,
    }
}

#[tauri::command]
pub async fn get_interface_details(
    device: String,
    database: State<'_, Database>,
) -> Result<Interface, String> {
    info!("Getting details for interface: {}", device);

    // Get all interfaces and filter for the requested one
    let interfaces = get_interfaces(database).await?;

    interfaces
        .into_iter()
        .find(|iface| iface.device == device)
        .ok_or_else(|| format!("Interface '{}' not found", device))
}
