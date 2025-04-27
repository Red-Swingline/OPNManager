use crate::db::Database;
use crate::http_client::make_http_request;
use serde_json::Value;
use std::time::{Duration, Instant};
use tauri::State;
use tokio::time::sleep;

fn build_api_url(api_info: &crate::db::ApiInfo, endpoint: &str) -> String {
    format!("{}:{}{}", api_info.api_url, api_info.port, endpoint)
}

#[tauri::command]
pub async fn check_for_updates(database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let check_url = build_api_url(&api_info, "/api/core/firmware/check");
    let check_response = make_http_request(
        "POST",
        &check_url,
        Some(serde_json::json!({})),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let check_body: Value = check_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse check response: {}", e))?;

    if check_body["status"] != "ok" {
        return Err(format!("Check failed: {:?}", check_body));
    }

    // Poll for check status
    let status_url = build_api_url(&api_info, "/api/core/firmware/upgradestatus");
    loop {
        let status_response = make_http_request(
            "GET",
            &status_url,
            None,
            None,
            Some(30),
            Some(&api_info.api_key),
            Some(&api_info.api_secret),
        )
        .await?;

        let status_body: Value = status_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse status response: {}", e))?;

        if status_body["status"] == "done" {
            break;
        }

        sleep(Duration::from_secs(2)).await;
    }

    let firmware_status_url = build_api_url(&api_info, "/api/core/firmware/status");
    let firmware_status_response = make_http_request(
        "GET",
        &firmware_status_url,
        None,
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let firmware_status: Value = firmware_status_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse firmware status: {}", e))?;

    let firmware_info_url = build_api_url(&api_info, "/api/core/firmware/info");
    let firmware_info_response = make_http_request(
        "GET",
        &firmware_info_url,
        None,
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let firmware_info: Value = firmware_info_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse firmware info: {}", e))?;

    let mut result = firmware_status.clone();

    // Add latest version info
    result["latest_version"] = firmware_info["product"]["product_latest"].clone();

    // Check for major upgrades (existing logic)
    if firmware_status["upgrade_major_version"].is_string() {
        result["has_major_upgrade"] = serde_json::json!(true);
        result["major_upgrade_version"] = firmware_status["upgrade_major_version"].clone();
        result["major_upgrade_message"] = firmware_status["upgrade_major_message"].clone();
    } else {
        result["has_major_upgrade"] = serde_json::json!(false);
    }

    // NEW: Check for minor upgrades in the upgrade_packages array
    if let Some(upgrade_packages) = firmware_status["upgrade_packages"].as_array() {
        if !upgrade_packages.is_empty() {
            // Find the main opnsense package upgrade
            for package in upgrade_packages {
                if package["name"] == "opnsense" {
                    result["has_minor_upgrade"] = serde_json::json!(true);
                    result["minor_upgrade_from"] = package["current_version"].clone();
                    result["minor_upgrade_to"] = package["new_version"].clone();

                    // Let's compute a semantic version string for frontend display
                    let new_version = package["new_version"].as_str().unwrap_or("");
                    if !new_version.is_empty() {
                        result["target_version"] = serde_json::json!(new_version);
                    }
                    break;
                }
            }
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_changelog(
    database: State<'_, Database>,
    version: String,
) -> Result<String, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let changelog_url = build_api_url(
        &api_info,
        &format!("/api/core/firmware/changelog/{}", version),
    );
    let response = make_http_request(
        "POST",
        &changelog_url,
        Some(serde_json::json!({})),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let changelog: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse changelog response: {}", e))?;

    Ok(changelog["html"].as_str().unwrap_or("").to_string())
}

#[tauri::command]
pub async fn start_update(database: State<'_, Database>) -> Result<String, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let update_url = build_api_url(&api_info, "/api/core/firmware/update");
    let response = make_http_request(
        "POST",
        &update_url,
        Some(serde_json::json!({})),
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let update_response: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse update response: {}", e))?;

    if update_response["status"] != "ok" {
        return Err(format!("Update failed: {:?}", update_response));
    }

    let status_url = build_api_url(&api_info, "/api/core/firmware/upgradestatus");
    let start_time = Instant::now();
    let timeout = Duration::from_secs(1800); // 30 minutes timeout
    let mut reboot_detected = false;

    while start_time.elapsed() < timeout {
        match make_http_request(
            "GET",
            &status_url,
            None,
            None,
            Some(5),
            Some(&api_info.api_key),
            Some(&api_info.api_secret),
        )
        .await
        {
            Ok(response) => {
                if reboot_detected {
                    return Ok("Update completed successfully. System is back online.".to_string());
                }

                let upgrade_status: Value = response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse upgrade status: {}", e))?;

                match upgrade_status["status"].as_str() {
                    Some("reboot") => {
                        println!("Reboot initiated, waiting for system to become unresponsive...");
                        reboot_detected = true;
                    }
                    Some("done") => {
                        if !reboot_detected {
                            println!("Update process completed, waiting for reboot...");
                        }
                    }
                    Some(status) => println!("Current status: {}", status),
                    None => println!("Unknown status"),
                }
            }
            Err(_) => {
                if reboot_detected {
                    println!("System is unresponsive, waiting for it to come back online...");
                } else {
                    reboot_detected = true;
                    println!("Lost connection to system, possible reboot in progress...");
                }
            }
        }

        sleep(Duration::from_secs(10)).await;
    }

    Err("Update timed out or failed to detect system coming back online".to_string())
}

#[tauri::command]
pub async fn get_current_firmware_status(database: State<'_, Database>) -> Result<Value, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let firmware_status_url = build_api_url(&api_info, "/api/core/firmware/status");
    let firmware_status_response = make_http_request(
        "GET",
        &firmware_status_url,
        None,
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let firmware_status: Value = firmware_status_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse firmware status: {}", e))?;

    let has_major_upgrade = firmware_status["upgrade_major_version"].is_string();

    let mut result = firmware_status.clone();

    // Handle major upgrades
    if has_major_upgrade {
        result["has_major_upgrade"] = serde_json::json!(true);
        result["major_upgrade_version"] = firmware_status["upgrade_major_version"].clone();
        result["major_upgrade_message"] = firmware_status["upgrade_major_message"].clone();
    } else {
        result["has_major_upgrade"] = serde_json::json!(false);
    }

    // Handle minor upgrades
    if let Some(upgrade_packages) = firmware_status["upgrade_packages"].as_array() {
        if !upgrade_packages.is_empty() {
            // Find the main opnsense package upgrade
            for package in upgrade_packages {
                if package["name"] == "opnsense" {
                    result["has_minor_upgrade"] = serde_json::json!(true);
                    result["minor_upgrade_from"] = package["current_version"].clone();
                    result["minor_upgrade_to"] = package["new_version"].clone();

                    // Set target version for frontend display
                    let new_version = package["new_version"].as_str().unwrap_or("");
                    if !new_version.is_empty() {
                        result["target_version"] = serde_json::json!(new_version);
                    }
                    break;
                }
            }
        }
    } else {
        result["has_minor_upgrade"] = serde_json::json!(false);
    }

    // Set status from the status field if available
    if firmware_status["status"].is_string() {
        let status = firmware_status["status"].as_str().unwrap_or("");
        if status == "update" {
            result["updates_available"] = serde_json::json!(true);
        } else {
            result["updates_available"] = serde_json::json!(false);
        }
    }

    Ok(result)
}
