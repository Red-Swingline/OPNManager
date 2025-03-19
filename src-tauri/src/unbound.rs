use crate::db::Database;
use crate::http_client::make_http_request;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnboundSettings {
    unbound: UnboundConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnboundConfig {
    general: Option<Value>,
    advanced: Option<Value>,
    acls: Option<Value>,
    dnsbl: Option<DnsblConfig>,
    forwarding: Option<Value>,
    dots: Option<Value>,
    hosts: Option<Value>,
    aliases: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DnsblConfig {
    enabled: String,
    safesearch: String,
    #[serde(rename = "type")]
    blocklist_type: Option<String>,
    lists: Option<Vec<String>>,
    whitelists: Option<Vec<String>>,
    blocklists: Option<Vec<String>>,
    wildcards: Option<Vec<String>>,
    address: Option<String>,
    nxdomain: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DnsblTypeOption {
    value: String,
    selected: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CronJob {
    uuid: Option<String>,
    enabled: String,
    minutes: String,
    hours: String,
    days: String,
    months: String,
    weekdays: String,
    description: String,
    command: String,
    parameters: Option<String>,
    origin: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CronJobsResponse {
    rows: Vec<CronJob>,
    rowCount: u32,
    total: u32,
    current: u32,
}

fn build_api_url(api_info: &crate::db::ApiInfo, endpoint: &str) -> String {
    format!("{}:{}{}", api_info.api_url, api_info.port, endpoint)
}

#[tauri::command]
pub async fn get_unbound_settings(database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/unbound/settings/get");

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

    // Get the response as text first
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to get response text: {}", e))?;

    // Log the first part of the response for debugging
    log::info!("Unbound API Response (first 200 chars): {}", &response_text.chars().take(200).collect::<String>());

    // Try to parse it as JSON
    let parsed = match serde_json::from_str::<Value>(&response_text) {
        Ok(json_value) => json_value,
        Err(e) => return Err(format!("Failed to parse response: {} - Response was: {}", e, &response_text.chars().take(500).collect::<String>())),
    };
    
    // If this is the first time viewing, check if there's an active selection
    if let Some(dnsbl) = parsed.get("unbound").and_then(|u| u.get("dnsbl")) {
        if let Some(type_obj) = dnsbl.get("type") {
            // Check if it's a string (comma-separated list)
            if type_obj.is_string() {
                // Already a comma-separated list, keep it as is
                let active_types = type_obj.as_str().unwrap().to_string();
                log::info!("Found pre-selected blocklist types: {}", active_types);
                
                // Create a mutable copy of the response to modify
                let mut modified = parsed.clone();
                
                // Set the active types in the dnsbl object
                if let Some(dnsbl_obj) = modified.get_mut("unbound").and_then(|u| u.get_mut("dnsbl")) {
                    dnsbl_obj["active_types"] = json!(active_types);
                }
                
                return Ok(modified);
            } 
            else if type_obj.is_object() {
                // Extract all active selections
                let mut active_types = Vec::new();
                for (key, value) in type_obj.as_object().unwrap() {
                    if let Some(selected) = value.get("selected") {
                        if selected.as_u64().unwrap_or(0) == 1 {
                            active_types.push(key.clone());
                        }
                    }
                }
                
                // If there are active selections, add them directly to the dnsbl object
                if !active_types.is_empty() {
                    let active_types_str = active_types.join(",");
                    log::info!("Found selected blocklist types: {}", active_types_str);
                    
                    // Create a mutable copy of the response to modify
                    let mut modified = parsed.clone();
                    
                    // Set the active types in the dnsbl object
                    if let Some(dnsbl_obj) = modified.get_mut("unbound").and_then(|u| u.get_mut("dnsbl")) {
                        dnsbl_obj["active_types"] = json!(active_types_str);
                    }
                    
                    return Ok(modified);
                }
            }
        }
    }
    
    Ok(parsed)
}

#[tauri::command]
pub async fn set_dnsbl_settings(
    database: State<'_, Database>,
    enabled: bool,
    safesearch: bool,
    blocklist_types: Vec<String>,
    lists: Vec<String>,
    whitelists: Vec<String>,
    blocklists: Vec<String>,
    wildcards: Vec<String>,
    address: String,
    nxdomain: bool,
) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/unbound/settings/set");

    // Build the DNSBL config payload
    let dnsbl_config = json!({
        "enabled": if enabled { "1" } else { "0" },
        "safesearch": if safesearch { "1" } else { "0" },
        "type": blocklist_types.join(","),
        "lists": lists.join(","),
        "whitelists": whitelists.join(","),
        "blocklists": blocklists.join(","),
        "wildcards": wildcards.join(","),
        "address": address,
        "nxdomain": if nxdomain { "1" } else { "0" }
    });

    let payload = json!({
        "unbound": {
            "dnsbl": dnsbl_config
        }
    });

    log::info!("Sending DNSBL settings: {}", serde_json::to_string(&payload).unwrap_or_default());

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

    // Get the response as text for better error handling
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to get response text: {}", e))?;

    // Log the response
    log::info!("DNSBL set response: {}", response_text);

    // Try to parse it as JSON
    match serde_json::from_str::<Value>(&response_text) {
        Ok(json_value) => Ok(json_value),
        Err(e) => Err(format!("Failed to parse response: {} - Response was: {}", e, response_text)),
    }
}

#[tauri::command]
pub async fn apply_dnsbl_settings(database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/unbound/service/dnsbl");

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

    response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

const UNBOUND_DNSBL_CRON_DESCRIPTION: &str = "OPNManager Unbound DNSBL Update";

#[tauri::command]
pub async fn get_dnsbl_cron_job(database: State<'_, Database>) -> Result<Option<CronJob>, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/cron/settings/searchJobs");

    let payload = json!({
        "current": 1,
        "rowCount": 1000,
        "sort": {},
        "searchPhrase": UNBOUND_DNSBL_CRON_DESCRIPTION
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

    let jobs_response = response
        .json::<CronJobsResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Find the cron job for Unbound DNSBL updates
    for job in jobs_response.rows {
        if job.description == UNBOUND_DNSBL_CRON_DESCRIPTION {
            return Ok(Some(job));
        }
    }

    Ok(None)
}

#[tauri::command]
pub async fn add_dnsbl_cron_job(
    database: State<'_, Database>,
    minutes: String,
    hours: String,
    days: String,
    months: String,
    weekdays: String,
) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    // First check if the job already exists
    let existing_job = get_dnsbl_cron_job(database.clone()).await?;
    if let Some(job) = existing_job {
        // Delete the existing job first
        if let Some(uuid) = job.uuid {
            delete_dnsbl_cron_job(database.clone(), uuid).await?;
        }
    }

    let url = build_api_url(&api_info, "/api/cron/settings/addJob/");

    let payload = json!({
        "job": {
            "enabled": "1",
            "minutes": minutes,
            "hours": hours,
            "days": days,
            "months": months,
            "weekdays": weekdays,
            "command": "unbound dnsbl",
            "parameters": "",
            "description": UNBOUND_DNSBL_CRON_DESCRIPTION
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

    let result = response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Apply the changes
    apply_cron_changes(database).await?;

    Ok(result)
}

#[tauri::command]
pub async fn delete_dnsbl_cron_job(
    database: State<'_, Database>,
    uuid: String,
) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, &format!("/api/cron/settings/delJob/{}", uuid));

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

    let result = response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Apply the changes
    apply_cron_changes(database).await?;

    Ok(result)
}

#[tauri::command]
pub async fn apply_cron_changes(database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/cron/service/reconfigure");

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

    response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}