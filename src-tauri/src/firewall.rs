use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use tauri::State;
use crate::db::Database;
use crate::http_client::make_http_request;

#[derive(Serialize, Deserialize, Debug)]
pub struct FirewallRule {
    uuid: String,
    enabled: String,
    sequence: String,
    description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirewallRulesResponse {
    rows: Vec<FirewallRule>,
    #[serde(rename = "rowCount")]
    row_count: u32,
    total: u32,
    current: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ToggleRuleResponse {
    result: String,
    changed: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApplyResponse {
    status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddRuleResponse {
    result: String,
    uuid: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkSelectOptions {
    pub aliases: Option<AliasOrNetwork>,
    pub networks: Option<AliasOrNetwork>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AliasOrNetwork {
    pub label: String,
    pub items: HashMap<String, String>,
}


fn build_api_url(api_info: &crate::db::ApiInfo, endpoint: &str) -> String {
    format!("{}:{}{}", api_info.api_url, api_info.port, endpoint)
}

#[tauri::command]
pub async fn get_firewall_rules(database: State<'_, Database>) -> Result<FirewallRulesResponse, String> {
    let api_info = database.get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/firewall/filter/searchRule");

    let payload = serde_json::json!({
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

    response.json::<FirewallRulesResponse>().await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn toggle_firewall_rule(database: State<'_, Database>, uuid: String) -> Result<ToggleRuleResponse, String> {
    let api_info = database.get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let toggle_url = build_api_url(&api_info, &format!("/api/firewall/filter/toggleRule/{}", uuid));

    let toggle_response = make_http_request(
        "POST",
        &toggle_url,
        Some(serde_json::json!({})),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    toggle_response.json::<ToggleRuleResponse>().await
        .map_err(|e| format!("Failed to parse toggle response: {}", e))
}

#[tauri::command]
pub async fn apply_firewall_changes(database: State<'_, Database>) -> Result<ApplyResponse, String> {
    let api_info = database.get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let apply_url = build_api_url(&api_info, "/api/firewall/filter/apply");

    let apply_response = make_http_request(
        "POST",
        &apply_url,
        Some(serde_json::json!({})),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    apply_response.json::<ApplyResponse>().await
        .map_err(|e| format!("Failed to parse apply response: {}", e))
}

#[tauri::command]
pub async fn get_rule_template(database: State<'_, Database>) -> Result<serde_json::Value, String> {
    let api_info = database.get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/firewall/filter/get_rule/");

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

    response.json::<serde_json::Value>().await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn add_firewall_rule(
    database: State<'_, Database>, 
    rule_data: serde_json::Value
) -> Result<AddRuleResponse, String> {
    let api_info = database.get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/firewall/filter/add_rule/");

    let response = make_http_request(
        "POST",
        &url,
        Some(rule_data),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let add_result = response.json::<AddRuleResponse>().await
        .map_err(|e| format!("Failed to parse add rule response: {}", e))?;
    
    // If the rule was added successfully, apply the changes
    if add_result.result == "saved" {
        apply_firewall_changes(database).await?;
    }

    Ok(add_result)
}

#[tauri::command]
pub async fn delete_firewall_rule(database: State<'_, Database>, uuid: String) -> Result<serde_json::Value, String> {
    let api_info = database.get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, &format!("/api/firewall/filter/del_rule/{}", uuid));

    let response = make_http_request(
        "POST",
        &url,
        Some(serde_json::json!({})),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let result = response.json::<serde_json::Value>().await
        .map_err(|e| format!("Failed to parse delete rule response: {}", e))?;
    
    // Apply the changes to make them take effect
    apply_firewall_changes(database).await?;
    
    Ok(result)
}


#[tauri::command]
pub async fn list_network_select_options(database: State<'_, Database>) -> Result<NetworkSelectOptions, String> {
    let api_info = database.get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/firewall/filter/list_network_select_options");

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

    response.json::<NetworkSelectOptions>().await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn get_rule(database: State<'_, Database>, uuid: String) -> Result<serde_json::Value, String> {
    let api_info = database.get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, &format!("/api/firewall/filter/get_rule/{}", uuid));

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

    response.json::<serde_json::Value>().await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn set_rule(
    database: State<'_, Database>, 
    uuid: String,
    rule_data: serde_json::Value
) -> Result<serde_json::Value, String> {
    let api_info = database.get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, &format!("/api/firewall/filter/set_rule/{}", uuid));

    let response = make_http_request(
        "POST",
        &url,
        Some(rule_data),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let result = response.json::<serde_json::Value>().await
        .map_err(|e| format!("Failed to parse set rule response: {}", e))?;
    
    // Apply the changes to make them take effect
    apply_firewall_changes(database).await?;
    
    Ok(result)
}