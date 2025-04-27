use crate::db::Database;
use crate::http_client::make_http_request;
use serde_json::{json, Value};
use tauri::State;

/// Search for tunables with pagination support
#[tauri::command]
pub async fn search_tunables(
    database: State<'_, Database>,
    current_page: u32,
    row_count: u32,
    search_phrase: String,
) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/core/tunables/search_item/",
        api_info.api_url, api_info.port
    );

    let payload = json!({
        "current": current_page,
        "rowCount": row_count,
        "sort": {},
        "searchPhrase": search_phrase
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

/// Get a specific tunable by UUID
#[tauri::command]
pub async fn get_tunable(database: State<'_, Database>, uuid: String) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/core/tunables/get_item/{}",
        api_info.api_url, api_info.port, uuid
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

/// Update a tunable setting
#[tauri::command]
pub async fn set_tunable(
    database: State<'_, Database>,
    uuid: String,
    tunable: String,
    value: String,
    description: String,
) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/core/tunables/set_item/{}",
        api_info.api_url, api_info.port, uuid
    );

    let payload = json!({
        "sysctl": {
            "tunable": tunable,
            "value": value,
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

/// Apply tunable changes
#[tauri::command]
pub async fn apply_tunables(database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/core/tunables/reconfigure",
        api_info.api_url, api_info.port
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

    response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Save and apply a tunable setting (convenience function)
#[tauri::command]
pub async fn save_and_apply_tunable(
    database: State<'_, Database>,
    uuid: String,
    tunable: String,
    value: String,
    description: String,
) -> Result<Value, String> {
    // First set the tunable
    let set_result = set_tunable(database.clone(), uuid, tunable, value, description).await?;

    // Check if save was successful
    if let Some(result) = set_result.get("result") {
        if result.as_str() == Some("saved") {
            // Then apply changes
            let apply_result = apply_tunables(database).await?;

            // Return combined result
            return Ok(json!({
                "set": set_result,
                "apply": apply_result
            }));
        }
    }

    // If we get here, the set operation didn't succeed
    Ok(json!({
        "error": "Failed to save tunable",
        "set_result": set_result
    }))
}

/// Add a new tunable
#[tauri::command]
pub async fn add_tunable(
    database: State<'_, Database>,
    tunable: String,
    value: String,
    description: String,
) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/core/tunables/add_item/",
        api_info.api_url, api_info.port
    );

    let payload = json!({
        "sysctl": {
            "tunable": tunable,
            "value": value,
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

/// Delete a tunable
#[tauri::command]
pub async fn delete_tunable(database: State<'_, Database>, uuid: String) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/core/tunables/del_item/{}",
        api_info.api_url, api_info.port, uuid
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

    response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}
