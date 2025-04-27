use std::collections::HashMap;

use crate::db::Database;
use crate::http_client::make_http_request;
use serde::{Deserialize, Serialize};
use tauri::State;

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
    uuid: Option<String>,
    // Include additional fields that may be in the response
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    validations: Option<serde_json::Value>,
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
pub async fn get_firewall_rules(
    database: State<'_, Database>,
) -> Result<FirewallRulesResponse, String> {
    let api_info = database
        .get_default_api_info()
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

    response
        .json::<FirewallRulesResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn toggle_firewall_rule(
    database: State<'_, Database>,
    uuid: String,
) -> Result<ToggleRuleResponse, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let toggle_url = build_api_url(
        &api_info,
        &format!("/api/firewall/filter/toggleRule/{}", uuid),
    );

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

    toggle_response
        .json::<ToggleRuleResponse>()
        .await
        .map_err(|e| format!("Failed to parse toggle response: {}", e))
}

#[tauri::command]
pub async fn apply_firewall_changes(
    database: State<'_, Database>,
) -> Result<ApplyResponse, String> {
    let api_info = database
        .get_default_api_info()
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

    apply_response
        .json::<ApplyResponse>()
        .await
        .map_err(|e| format!("Failed to parse apply response: {}", e))
}

#[tauri::command]
pub async fn get_rule_template(database: State<'_, Database>) -> Result<serde_json::Value, String> {
    let api_info = database
        .get_default_api_info()
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

    response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn add_firewall_rule(
    database: State<'_, Database>,
    rule_data: serde_json::Value,
) -> Result<AddRuleResponse, String> {
    let api_info = database
        .get_default_api_info()
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

    // First try to parse as AddRuleResponse
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    // Log the raw response for debugging
    println!("Raw add rule response: {}", response_text);

    // Try to parse as AddRuleResponse
    let add_result = match serde_json::from_str::<AddRuleResponse>(&response_text) {
        Ok(result) => result,
        Err(e) => {
            // If parsing fails, try to parse as a generic JSON value
            match serde_json::from_str::<serde_json::Value>(&response_text) {
                Ok(value) => {
                    // Try to extract result field
                    let result = value
                        .get("result")
                        .and_then(|v| v.as_str())
                        .unwrap_or("error")
                        .to_string();

                    // Create a default response
                    AddRuleResponse {
                        result,
                        uuid: None,
                        status: None,
                        validations: Some(value),
                    }
                }
                Err(_) => {
                    // If all parsing fails, return a meaningful error
                    return Err(format!("Failed to parse API response: {}", response_text));
                }
            }
        }
    };

    // Apply changes only if the result was successful
    if add_result.result == "saved" {
        apply_firewall_changes(database).await?;
    }

    Ok(add_result)
}

#[tauri::command]
pub async fn delete_firewall_rule(
    database: State<'_, Database>,
    uuid: String,
) -> Result<serde_json::Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(
        &api_info,
        &format!("/api/firewall/filter/del_rule/{}", uuid),
    );

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

    let result = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse delete rule response: {}", e))?;

    apply_firewall_changes(database).await?;

    Ok(result)
}

#[tauri::command]
pub async fn list_network_select_options(
    database: State<'_, Database>,
) -> Result<NetworkSelectOptions, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(
        &api_info,
        "/api/firewall/filter/list_network_select_options",
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
        .json::<NetworkSelectOptions>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn get_rule(
    database: State<'_, Database>,
    uuid: String,
) -> Result<serde_json::Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(
        &api_info,
        &format!("/api/firewall/filter/get_rule/{}", uuid),
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
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn set_rule(
    database: State<'_, Database>,
    uuid: String,
    rule_data: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(
        &api_info,
        &format!("/api/firewall/filter/set_rule/{}", uuid),
    );

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

    let result = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse set rule response: {}", e))?;

    apply_firewall_changes(database).await?;

    Ok(result)
}
