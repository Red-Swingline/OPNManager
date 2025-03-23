use crate::db::{self, ApiInfo, Database};
use crate::http_client::make_http_request;
use crate::pin_cache::PinCache;
use log::{error, info};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub async fn get_vendor_info(mac: String) -> Result<String, String> {
    let formatted_mac = mac.replace(":", "-");
    let url = format!("https://api.macvendors.com/{}", formatted_mac);

    match make_http_request("GET", &url, None, None, Some(30), None, None).await {
        Ok(response) => {
            if response.status().is_success() {
                Ok(response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown".to_string()))
            } else if response.status().as_u16() == 404 {
                Ok("Unknown".to_string())
            } else {
                Err(format!("Failed to get vendor info: {}", response.status()))
            }
        }
        Err(e) => Err(format!("Failed to get vendor info: {}", e)),
    }
}

#[tauri::command]
pub fn check_first_run(database: State<Database>) -> Result<bool, String> {
    database.is_first_run().map_err(|e| e.to_string())
}

#[derive(Deserialize)]
pub struct InitialConfig {
    profile_name: String,
    api_key: String,
    api_secret: String,
    api_url: String,
    port: u16,
    pin: String,
}

#[tauri::command]
pub async fn save_initial_config(
    config: InitialConfig,
    database: State<'_, Database>,
    pin_cache: State<'_, PinCache>,
) -> Result<(), String> {
    info!("Starting save_initial_config");

    info!("Hashing password");
    let password_hash = Database::hash_password(&config.pin).map_err(|e| {
        error!("Failed to hash password: {}", e);
        format!("Failed to hash password: {}", e)
    })?;

    info!("Updating password hash");
    database.update_password_hash(&password_hash).map_err(|e| {
        error!("Failed to save password hash: {}", e);
        format!("Failed to save password hash: {}", e)
    })?;

    pin_cache.set_pin(config.pin.clone());

    info!("Creating ApiInfo");
    let api_info = ApiInfo {
        id: 0,
        profile_name: config.profile_name,
        api_key: config.api_key,
        api_secret: config.api_secret,
        api_url: config.api_url,
        port: config.port,
        is_default: true,
    };

    info!("Saving API info");
    database.save_initial_api_info(&api_info).map_err(|e| {
        error!("Failed to save API info: {}", e);
        format!("Failed to save API info: {}", e)
    })?;

    info!("API info saved successfully");

    info!("Setting has_run flag");
    database.set_has_run().map_err(|e| {
        error!("Failed to set has_run flag: {}", e);
        format!("Failed to set has_run flag: {}", e)
    })?;

    info!("Has_run flag set successfully");
    info!("save_initial_config completed successfully");
    Ok(())
}

#[tauri::command]
pub fn get_api_info(database: State<Database>) -> Result<Option<ApiInfo>, String> {
    database.get_default_api_info().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_api_profiles(database: State<Database>) -> Result<Vec<ApiInfo>, String> {
    database
        .list_api_profiles()
        .map_err(|e| format!("Failed to get API profiles: {}", e))
}

#[tauri::command]
pub fn update_api_info(
    profile_name: String,
    api_key: String,
    api_secret: String,
    api_url: String,
    port: u16,
    is_default: bool,
    database: State<Database>,
) -> Result<(), String> {
    let mut api_info = database
        .get_api_info(Some(&profile_name))
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("API profile '{}' not found", profile_name))?;

    api_info.api_key = api_key;
    api_info.api_secret = api_secret;
    api_info.api_url = api_url;
    api_info.port = port;
    api_info.is_default = is_default;

    database
        .save_api_info(&api_info)
        .map_err(|e| e.to_string())?;

    if is_default {
        database
            .set_default_profile(&profile_name)
            .map_err(|e| format!("Failed to set default profile: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn update_pin(
    current_pin: String,
    new_pin: String,
    confirm_new_pin: String,
    database: State<Database>,
    pin_cache: State<PinCache>,
) -> Result<(), String> {
    if new_pin != confirm_new_pin {
        return Err("New PIN and confirmation do not match".to_string());
    }

    let result = database.update_pin(&current_pin, &new_pin)?;
    pin_cache.set_pin(new_pin);

    Ok(())
}

#[derive(Deserialize)]
pub struct NewApiProfile {
    profile_name: String,
    api_key: String,
    api_secret: String,
    api_url: String,
    port: u16,
}

#[tauri::command]
pub async fn add_api_profile(
    profile: NewApiProfile,
    database: State<'_, Database>,
) -> Result<(), String> {
    info!("Starting add_api_profile");

    let api_info = ApiInfo {
        id: 0,
        profile_name: profile.profile_name,
        api_key: profile.api_key,
        api_secret: profile.api_secret,
        api_url: profile.api_url,
        port: profile.port,
        is_default: false,
    };

    info!("Saving new API profile");
    database.save_api_info(&api_info).map_err(|e| {
        error!("Failed to save API profile: {}", e);
        format!("Failed to save API profile: {}", e)
    })?;

    info!("New API profile saved successfully");
    Ok(())
}

#[tauri::command]
pub fn delete_api_profile(profile_name: String, database: State<Database>) -> Result<(), String> {
    info!("Starting delete_api_profile for profile: {}", profile_name);

    let profiles = database
        .list_api_profiles()
        .map_err(|e| format!("Failed to list API profiles: {}", e))?;

    if profiles.len() == 1 {
        return Err("Cannot delete the last profile".to_string());
    }

    let is_default = profiles
        .iter()
        .find(|p| p.profile_name == profile_name)
        .map(|p| p.is_default)
        .unwrap_or(false);

    database.delete_api_profile(&profile_name).map_err(|e| {
        error!("Failed to delete API profile: {}", e);
        format!("Failed to delete API profile: {}", e)
    })?;

    if is_default {
        let new_default = profiles
            .iter()
            .find(|p| p.profile_name != profile_name)
            .ok_or_else(|| "No other profile found to set as default".to_string())?;

        database
            .set_default_profile(&new_default.profile_name)
            .map_err(|e| {
                error!("Failed to set new default profile: {}", e);
                format!("Failed to set new default profile: {}", e)
            })?;
    }

    info!("API profile deleted successfully");
    Ok(())
}

#[tauri::command]
pub fn set_default_profile(profile_name: String, database: State<Database>) -> Result<(), String> {
    info!("Setting default profile: {}", profile_name);
    database.set_default_profile(&profile_name).map_err(|e| {
        error!("Failed to set default profile: {}", e);
        format!("Failed to set default profile: {}", e)
    })
}

#[tauri::command]
pub async fn test_api_connection(
    api_key: String,
    api_secret: String,
    api_url: String,
    port: u16,
) -> Result<bool, String> {
    // Build the API URL for a simple endpoint (system information is a good test)
    let url = format!("{}:{}/api/diagnostics/system/systemTime", api_url, port);

    // Make the request using the provided credentials
    let response = make_http_request(
        "GET",
        &url,
        None,
        None,
        Some(10), // shorter timeout for testing
        Some(&api_key),
        Some(&api_secret),
    )
    .await;

    match response {
        Ok(resp) => {
            // Try to parse the response as JSON to verify it's valid
            match resp.json::<Value>().await {
                Ok(_) => Ok(true), // Successfully connected and got valid JSON
                Err(e) => Err(format!(
                    "Connection succeeded but returned invalid data: {}",
                    e
                )),
            }
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub fn get_dashboard_preferences(
    database: State<Database>,
) -> Result<HashMap<String, db::DashboardWidgetPref>, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    database
        .get_dashboard_preferences(api_info.id)
        .map_err(|e| format!("Failed to get dashboard preferences: {}", e))
}

#[tauri::command]
pub fn save_dashboard_preferences(
    prefs: Vec<db::DashboardWidgetPref>,
    database: State<Database>,
) -> Result<(), String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    database
        .save_dashboard_preferences(api_info.id, &prefs)
        .map_err(|e| format!("Failed to save dashboard preferences: {}", e))
}
