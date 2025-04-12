use crate::db::Database;
use crate::http_client::make_http_request;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

#[derive(Serialize, Deserialize, Debug)]
pub struct Route {
    uuid: String,
    disabled: String,
    network: String,
    gateway: String,
    descr: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoutesResponse {
    rows: Vec<Route>,
    #[serde(rename = "rowCount")]
    row_count: u32,
    total: u32,
    current: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GatewayOption {
    value: String,
    selected: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RouteInfo {
    network: String,
    gateway: std::collections::HashMap<String, GatewayOption>,
    descr: String,
    disabled: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RouteInfoResponse {
    route: RouteInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddRouteResponse {
    result: String,
    uuid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ToggleResponse {
    result: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReconfigureResponse {
    status: String,
}

fn build_api_url(api_info: &crate::db::ApiInfo, endpoint: &str) -> String {
    format!("{}:{}{}", api_info.api_url, api_info.port, endpoint)
}

#[tauri::command]
pub async fn get_routes(database: State<'_, Database>) -> Result<RoutesResponse, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/routes/routes/searchroute");

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
        .json::<RoutesResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn get_route_info(database: State<'_, Database>) -> Result<RouteInfoResponse, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/routes/routes/getroute");

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
        .json::<RouteInfoResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn add_route(
    database: State<'_, Database>,
    network: String,
    gateway: String,
    description: String,
    disabled: bool,
) -> Result<AddRouteResponse, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/routes/routes/addroute");

    let payload = json!({
        "route": {
            "disabled": if disabled { "1" } else { "0" },
            "network": network,
            "gateway": gateway,
            "descr": description
        }
    });

    println!("Sending payload: {:?}", payload);

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

    println!("Received response: {}", response_text);

    let result = serde_json::from_str(&response_text).map_err(|e| {
        format!(
            "Failed to parse response: {} (Response was: {})",
            e, response_text
        )
    })?;

    apply_changes(database).await?;

    Ok(result)
}

#[tauri::command]
pub async fn delete_route(database: State<'_, Database>, uuid: String) -> Result<(), String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, &format!("/api/routes/routes/delroute/{}", uuid));

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

    if !response.status().is_success() {
        return Err(format!("Failed to delete route: {}", response.status()));
    }

    apply_changes(database).await?;

    Ok(())
}

#[tauri::command]
pub async fn toggle_route(
    database: State<'_, Database>,
    uuid: String,
) -> Result<ToggleResponse, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(
        &api_info,
        &format!("/api/routes/routes/toggleroute/{}", uuid),
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
        .json::<ToggleResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    apply_changes(database).await?;

    Ok(result)
}

#[tauri::command]
pub async fn apply_changes(database: State<'_, Database>) -> Result<ReconfigureResponse, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = build_api_url(&api_info, "/api/routes/routes/reconfigure");

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
        .json::<ReconfigureResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}
