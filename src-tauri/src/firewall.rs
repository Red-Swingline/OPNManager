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
    #[serde(default)]
    interface: Option<String>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct InterfaceGroup {
    pub label: String,
    pub icon: String,
    pub items: Vec<InterfaceItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InterfaceItem {
    pub value: String,
    pub label: String,
    #[serde(default)]
    pub selected: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InterfaceListResponse {
    pub floating: InterfaceGroup,
    pub groups: InterfaceGroup,
    pub interfaces: InterfaceGroup,
}

fn build_api_url(api_info: &crate::db::ApiInfo, endpoint: &str) -> String {
    format!("{}:{}{}", api_info.api_url, api_info.port, endpoint)
}

#[tauri::command]
pub async fn get_interface_list(
    database: State<'_, Database>,
) -> Result<InterfaceListResponse, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/firewall/filter/get_interface_list");

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
        .json::<InterfaceListResponse>()
        .await
        .map_err(|e| format!("Failed to parse interface list response: {}", e))
}

#[tauri::command]
pub async fn check_api_version(
    database: State<'_, Database>,
) -> Result<bool, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;
    
    let url = build_api_url(&api_info, "/api/firewall/filter/get_interface_list");
    
    let response = make_http_request(
        "GET",
        &url,
        None,
        None,
        Some(10), 
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    ).await;
 
    Ok(response.is_ok())
}

#[tauri::command]
pub async fn get_firewall_rules(
    database: State<'_, Database>,
    interface: Option<String>,
) -> Result<FirewallRulesResponse, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let is_new_api = check_api_version(database.clone()).await.unwrap_or(false);

    let url = build_api_url(&api_info, "/api/firewall/filter/search_rule");

    let mut payload = serde_json::json!({
        "current": 1,
        "rowCount": -1,
        "sort": {},
        "searchPhrase": ""
    });

    if let Some(iface) = interface {
        if !iface.is_empty() {
            payload["interface"] = serde_json::Value::String(iface);
        }
    }
    
    println!("Getting firewall rules from URL: {}", url);
    println!("With payload: {}", serde_json::to_string_pretty(&payload).unwrap_or_default());

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

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    println!("Raw add rule response: {}", response_text);

    let add_result = match serde_json::from_str::<AddRuleResponse>(&response_text) {
        Ok(result) => result,
        Err(e) => {
            match serde_json::from_str::<serde_json::Value>(&response_text) {
                Ok(value) => {
                    let result = value
                        .get("result")
                        .and_then(|v| v.as_str())
                        .unwrap_or("error")
                        .to_string();

                    AddRuleResponse {
                        result,
                        uuid: None,
                        status: None,
                        validations: Some(value),
                    }
                }
                Err(_) => {
                    return Err(format!("Failed to parse API response: {}", response_text));
                }
            }
        }
    };

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
    .await;

    match response {
        Ok(resp) => {
            resp.json::<serde_json::Value>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        },
        Err(e) => {
            Err(format!("Failed to get rule: {}", e))
        }
    }
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

    let is_new_api = check_api_version(database.clone()).await.unwrap_or(false);

    let url = build_api_url(
        &api_info,
        &format!("/api/firewall/filter/set_rule/{}", uuid),
    );

    let actual_payload = match rule_data {
        serde_json::Value::Object(ref map) => {
            if map.contains_key("rule") {
                rule_data.clone()
            } else {
                serde_json::json!({ "rule": map })
            }
        },
        _ => serde_json::json!({ "rule": rule_data }) 
    };

    println!("Setting rule {} with URL: {}", uuid, url);
    println!("Raw payload: {}", serde_json::to_string_pretty(&rule_data).unwrap_or_default());
    println!("Actual payload sent: {}", serde_json::to_string_pretty(&actual_payload).unwrap_or_default());

    let response = make_http_request(
        "POST",
        &url,
        Some(actual_payload),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    println!("Set rule response: {}", response_text);

    let result = match serde_json::from_str::<serde_json::Value>(&response_text) {
        Ok(value) => value,
        Err(e) => {
            return Err(format!("Failed to parse set rule response as JSON: {}. Raw response: {}", e, response_text));
        }
    };

    if let Some(result_field) = result.get("result") {
        if result_field.as_str() == Some("saved") {
            apply_firewall_changes(database).await?;
        }
    }

    Ok(result)
}
