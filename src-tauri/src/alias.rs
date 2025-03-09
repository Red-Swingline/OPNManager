use crate::db::Database;
use crate::http_client::make_http_request;
use serde_json::json;
use serde_json::Value;
use tauri::State;

fn build_api_url(api_info: &crate::db::ApiInfo, endpoint: &str) -> String {
    format!("{}:{}{}", api_info.api_url, api_info.port, endpoint)
}

#[tauri::command]
pub async fn list_network_aliases(database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/firewall/alias/listNetworkAliases");

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

#[tauri::command]
pub async fn get_alias(database: State<'_, Database>, alias_name: String) -> Result<Value, String> {
    let aliases = search_alias_items(database).await?;

    let alias = aliases["rows"]
        .as_array()
        .and_then(|rows| {
            rows.iter()
                .find(|row| row["name"].as_str() == Some(&alias_name))
        })
        .ok_or_else(|| format!("Alias '{}' not found", alias_name))?;

    Ok(alias.clone())
}

#[tauri::command]
pub async fn add_alias(
    database: State<'_, Database>,
    name: String,
    alias_type: String,
    content: String,
    description: String,
    enabled: bool,
) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/firewall/alias/addItem/");

    let formatted_content = content
        .split(',')
        .map(|s| s.trim())
        .collect::<Vec<&str>>()
        .join("\n");

    let payload = json!({
        "alias": {
            "enabled": if enabled { "1" } else { "0" },
            "name": name,
            "type": alias_type,
            "proto": "",
            "categories": "",
            "updatefreq": "",
            "content": formatted_content,
            "interface": "",
            "counters": "0",
            "description": description
        },
        "network_content": "",
        "authgroup_content": ""
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

    if result["result"].as_str() == Some("saved") {
        apply_alias_changes(database).await?;
    }

    Ok(result)
}

#[tauri::command]
pub async fn add_ip_to_alias(
    database: State<'_, Database>,
    uuid: String,
    current_content: String,
    _new_ip: String,
) -> Result<(), String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, &format!("/api/firewall/alias/setItem/{}", uuid));

    let alias_info = get_alias_info(&api_info, &uuid).await?;
    let alias_name = alias_info["alias"]["name"].as_str().unwrap_or("");

    let payload = json!({
        "alias": {
            "name": alias_name,
            "content": current_content,
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

    if response.status().is_success() {
        apply_alias_changes(database).await?;
        Ok(())
    } else {
        Err(format!("Failed to add IP to alias: {}", response.status()))
    }
}

#[tauri::command]
pub async fn remove_ip_from_alias(
    database: State<'_, Database>,
    uuid: String,
    current_content: String,
) -> Result<(), String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, &format!("/api/firewall/alias/setItem/{}", uuid));

    let alias_info = get_alias_info(&api_info, &uuid).await?;
    let alias_name = alias_info["alias"]["name"].as_str().unwrap_or("");

    let payload = json!({
        "alias": {
            "name": alias_name,
            "content": current_content,
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

    if response.status().is_success() {
        apply_alias_changes(database).await?;
        Ok(())
    } else {
        Err(format!(
            "Failed to remove IP from alias: {}",
            response.status()
        ))
    }
}

#[tauri::command]
pub async fn toggle_alias(database: State<'_, Database>, uuid: String) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(
        &api_info,
        &format!("/api/firewall/alias/toggleItem/{}", uuid),
    );

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

    if result["changed"].as_bool().unwrap_or(false) {
        apply_alias_changes(database).await?;
    }

    Ok(result)
}

#[tauri::command]
pub async fn delete_alias(database: State<'_, Database>, uuid: String) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, &format!("/api/firewall/alias/delItem/{}", uuid));

    let response = make_http_request(
        "POST",
        &url,
        Some(json!({})),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await;

    match response {
        Ok(response) => {
            if response.status().is_success() {
                let result = response
                    .json::<Value>()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

                apply_alias_changes(database).await?;
                Ok(result)
            } else {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                if error_text.contains("Alias in use") || error_text.contains("Currently in use by")
                {
                    Err(format!("Cannot delete this alias because it is currently in use by firewall rules. Please remove references to this alias in your firewall rules first."))
                } else {
                    Err(format!("Server returned error: {}", error_text))
                }
            }
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn apply_alias_changes(database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/firewall/alias/set");

    let payload = json!({
        "alias": {
            "geoip": {
                "url": ""
            }
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

async fn get_alias_info(api_info: &crate::db::ApiInfo, uuid: &str) -> Result<Value, String> {
    let url = build_api_url(api_info, &format!("/api/firewall/alias/getItem/{}", uuid));

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

#[tauri::command]
pub async fn search_alias_items(database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/firewall/alias/searchItem");

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
