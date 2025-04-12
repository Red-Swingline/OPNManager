use crate::db::Database;
use crate::http_client::make_http_request;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize, Debug)]
pub struct Memory {
    total: String,
    total_frmt: String,
    used: u64,
    used_frmt: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemResources {
    memory: Memory,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiskDevice {
    device: String,
    #[serde(rename = "type")]
    device_type: String,
    blocks: String,
    used: String,
    available: String,
    used_pct: u8,
    mountpoint: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemDisk {
    devices: Vec<DiskDevice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TemperatureSensor {
    device: String,
    device_seq: String,
    temperature: String,
    #[serde(rename = "type")]
    sensor_type: String,
    type_translated: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemTemperature {
    sensors: Vec<TemperatureSensor>,
}

#[tauri::command]
pub async fn get_system_resources(
    database: State<'_, Database>,
) -> Result<SystemResources, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/diagnostics/system/systemResources",
        api_info.api_url, api_info.port
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
        .json::<SystemResources>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn get_system_disk(database: State<'_, Database>) -> Result<SystemDisk, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/diagnostics/system/systemDisk",
        api_info.api_url, api_info.port
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
        .json::<SystemDisk>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_system_temperature(
    database: State<'_, Database>,
) -> Result<SystemTemperature, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/diagnostics/system/systemTemperature",
        api_info.api_url, api_info.port
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

    // Handle empty array response for systems without temperature sensors
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to get response text: {}", e))?;

    // Log the actual response for debugging
    log::info!("Temperature API response: {}", response_text);

    if response_text.trim() == "[]" {
        return Ok(SystemTemperature {
            sensors: Vec::new(),
        });
    }

    // The API response is an array of sensor objects, but we need to adapt it to our structure
    match serde_json::from_str::<serde_json::Value>(&response_text) {
        Ok(json_value) => {
            if let Some(array) = json_value.as_array() {
                let mut sensors = Vec::new();

                for item in array {
                    // Extract fields from each sensor object
                    let device = item
                        .get("device")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    let device_seq = item
                        .get("device_seq")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    let temperature = item
                        .get("temperature")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    let sensor_type = item
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    let sensor_type_translated = item
                        .get("type_translated")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();

                    sensors.push(TemperatureSensor {
                        device: device.to_string(),
                        device_seq: device_seq.to_string(),
                        temperature: temperature.to_string(),
                        sensor_type: sensor_type.to_string(),
                        type_translated: sensor_type_translated.to_string(),
                    });
                }

                log::info!("Successfully parsed {} temperature sensors", sensors.len());
                Ok(SystemTemperature { sensors })
            } else {
                log::error!("Temperature API response is not an array");
                Ok(SystemTemperature {
                    sensors: Vec::new(),
                })
            }
        }
        Err(e) => {
            log::error!("Failed to parse temperature response: {}", e);
            Ok(SystemTemperature {
                sensors: Vec::new(),
            })
        }
    }
}
