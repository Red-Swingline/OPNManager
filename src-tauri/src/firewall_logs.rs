use crate::db::Database;
use crate::http_client::make_http_request;
use log::error;
use reqwest::header::{HeaderMap, ACCEPT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager, State, Window};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FirewallLog {
    rulenr: Option<String>,
    subrulenr: Option<String>,
    anchorname: Option<String>,
    rid: Option<String>,
    interface: Option<String>,
    reason: Option<String>,
    action: Option<String>,
    dir: Option<String>,
    ipversion: Option<String>,
    tos: Option<String>,
    ecn: Option<String>,
    ttl: Option<String>,
    id: Option<String>,
    offset: Option<String>,
    ipflags: Option<String>,
    protonum: Option<String>,
    protoname: Option<String>,
    length: Option<String>,
    src: Option<String>,
    dst: Option<String>,
    srcport: Option<String>,
    dstport: Option<String>,
    datalen: Option<String>,
    tcpflags: Option<String>,
    seq: Option<String>,
    ack: Option<String>,
    urp: Option<String>,
    tcpopts: Option<String>,
    #[serde(rename = "__timestamp__")]
    timestamp: Option<String>,
    #[serde(rename = "__host__")]
    host: Option<String>,
    #[serde(rename = "__digest__")]
    digest: Option<String>,
    #[serde(rename = "__spec__")]
    spec: Option<Vec<String>>,
    label: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogFilters {
    action: Vec<String>,
    interface_name: Vec<String>,
    dir: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InterfaceNames(pub HashMap<String, String>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogFilterCriteria {
    action: String,
    interface: String,
    direction: String,
    limit: usize,
}

pub struct LogCache {
    logs: Vec<FirewallLog>,
    last_digest: String,
    last_update: Instant,
    active_listeners: usize,
    filter_criteria: LogFilterCriteria,
}

impl LogCache {
    pub fn new() -> Self {
        Self {
            logs: Vec::with_capacity(500),
            last_digest: String::new(),
            last_update: Instant::now(),
            active_listeners: 0,
            filter_criteria: LogFilterCriteria {
                action: String::new(),
                interface: String::new(),
                direction: String::new(),
                limit: 500,
            },
        }
    }
}

pub fn register_log_cache(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(Arc::new(Mutex::new(LogCache::new())));
    Ok(())
}

#[tauri::command]
pub async fn get_log_filters(database: State<'_, Database>) -> Result<LogFilters, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/diagnostics/firewall/log_filters",
        api_info.api_url, api_info.port
    );

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".parse().unwrap());

    let response = make_http_request(
        "GET",
        &url,
        None,
        Some(headers),
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<LogFilters>()
        .await
        .map_err(|e| format!("Failed to parse log filters: {}", e))
}

#[tauri::command]
pub async fn get_interface_names(database: State<'_, Database>) -> Result<InterfaceNames, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/diagnostics/interface/getInterfaceNames",
        api_info.api_url, api_info.port
    );

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".parse().unwrap());

    let response = make_http_request(
        "GET",
        &url,
        None,
        Some(headers),
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<InterfaceNames>()
        .await
        .map_err(|e| format!("Failed to parse interface names: {}", e))
}

async fn fetch_firewall_logs(
    database: State<'_, Database>,
    digest: &str,
) -> Result<Vec<FirewallLog>, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/diagnostics/firewall/log/?digest={}&limit=500",
        api_info.api_url, api_info.port, digest
    );

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".parse().unwrap());

    let response = make_http_request(
        "GET",
        &url,
        None,
        Some(headers),
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to get response text: {}", e))?;

    match serde_json::from_str::<Vec<FirewallLog>>(&response_text) {
        Ok(logs) => Ok(logs),
        Err(e) => {
            error!("Failed to parse logs: {}", e);
            error!(
                "First 1000 chars of response: {}",
                &response_text.chars().take(1000).collect::<String>()
            );
            Err(format!(
                "Failed to parse logs: {}. First 1000 chars of response: {}",
                e,
                &response_text.chars().take(1000).collect::<String>()
            ))
        }
    }
}

#[tauri::command]
pub async fn get_firewall_logs(
    database: State<'_, Database>,
    log_cache: State<'_, Arc<Mutex<LogCache>>>,
) -> Result<Vec<FirewallLog>, String> {
    let digest;
    {
        let cache = log_cache.lock().unwrap();
        digest = cache.last_digest.clone();
    }
    let new_logs = fetch_firewall_logs(database, &digest).await?;

    let mut cache = log_cache.lock().unwrap();

    if !new_logs.is_empty() {
        for log in &new_logs {
            cache.logs.push(log.clone());
        }
        cache.logs.sort_by(|a, b| {
            let date_a = a.timestamp.as_ref().map_or(0, |ts| {
                chrono::DateTime::parse_from_rfc3339(ts)
                    .map(|dt| dt.timestamp())
                    .unwrap_or(0)
            });
            let date_b = b.timestamp.as_ref().map_or(0, |ts| {
                chrono::DateTime::parse_from_rfc3339(ts)
                    .map(|dt| dt.timestamp())
                    .unwrap_or(0)
            });
            date_b.cmp(&date_a)
        });
        // Use the digest from the latest log to avoid repeating requests for the same logs
        if let Some(last_log) = new_logs.last() {
            if let Some(digest) = &last_log.digest {
                cache.last_digest = digest.clone();
            }
        }
        // Keep fewer logs in memory (limit * 1.5 instead of limit * 2)
        if cache.logs.len() > cache.filter_criteria.limit * 3 / 2 {
            cache.logs = cache
                .logs
                .iter()
                .take(cache.filter_criteria.limit)
                .cloned()
                .collect();
        }

        cache.last_update = Instant::now();
    }
    let filtered_logs = cache
        .logs
        .iter()
        .filter(|log| {
            (cache.filter_criteria.action.is_empty()
                || log
                    .action
                    .as_ref()
                    .is_some_and(|a| a == &cache.filter_criteria.action))
                && (cache.filter_criteria.interface.is_empty()
                    || log
                        .interface
                        .as_ref()
                        .is_some_and(|i| i == &cache.filter_criteria.interface))
                && (cache.filter_criteria.direction.is_empty()
                    || log
                        .dir
                        .as_ref()
                        .is_some_and(|d| d == &cache.filter_criteria.direction))
        })
        .cloned()
        .collect::<Vec<_>>();

    Ok(filtered_logs
        .into_iter()
        .take(cache.filter_criteria.limit)
        .collect())
}

#[tauri::command]
pub fn update_log_filters(
    log_cache: State<'_, Arc<Mutex<LogCache>>>,
    action: String,
    interface: String,
    direction: String,
    limit: Option<usize>,
) -> Result<(), String> {
    let mut cache = log_cache.lock().unwrap();

    cache.filter_criteria = LogFilterCriteria {
        action,
        interface,
        direction,
        limit: limit.unwrap_or(1000),
    };

    Ok(())
}

#[tauri::command]
pub fn start_log_polling(
    window: Window,
    log_cache: State<'_, Arc<Mutex<LogCache>>>,
) -> Result<(), String> {
    {
        let mut cache = log_cache.lock().unwrap();
        cache.active_listeners += 1;
    }

    // Create new scoped identifiers to avoid name conflicts
    let log_cache_clone = log_cache.inner().clone();
    let window_clone = window.clone();
    let window_label = window.label().to_string();

    let _poll_task = tauri::async_runtime::spawn(async move {
        let database = window_clone.state::<Database>();

        // Track consecutive empty responses to dynamically adjust polling rate
        let mut consecutive_empty_responses = 0;
        let mut poll_interval_ms = 1000; // Start with 1 second polling
        let min_poll_interval_ms = 1000; // Never poll faster than once per second
        let max_poll_interval_ms = 5000; // Never wait more than 5 seconds between polls

        loop {
            let active_listeners;
            let digest;

            // Scope the mutex lock to minimize lock duration
            {
                let cache = log_cache_clone.lock().unwrap();
                active_listeners = cache.active_listeners;
                digest = cache.last_digest.clone();
            }

            // Check if we should stop polling
            if active_listeners == 0 {
                log::info!(
                    "No active listeners, stopping polling for window {}",
                    window_label
                );
                break;
            }

            // Fetch new logs using the latest digest
            match fetch_firewall_logs(database.clone(), &digest).await {
                Ok(new_logs) => {
                    if !new_logs.is_empty() {
                        // We have new logs, process them
                        let mut cache = log_cache_clone.lock().unwrap();

                        // Reset consecutive empty counter and poll interval since we got new logs
                        consecutive_empty_responses = 0;
                        poll_interval_ms = min_poll_interval_ms;

                        for log in &new_logs {
                            cache.logs.push(log.clone());
                        }

                        // Use the digest from the latest log to avoid repeating requests for the same logs
                        if let Some(last_log) = new_logs.last() {
                            if let Some(digest) = &last_log.digest {
                                cache.last_digest = digest.clone();
                            }
                        }

                        // Sort logs by timestamp (newest first)
                        cache.logs.sort_by(|a, b| {
                            let date_a = a.timestamp.as_ref().map_or(0, |ts| {
                                chrono::DateTime::parse_from_rfc3339(ts)
                                    .map(|dt| dt.timestamp())
                                    .unwrap_or(0)
                            });
                            let date_b = b.timestamp.as_ref().map_or(0, |ts| {
                                chrono::DateTime::parse_from_rfc3339(ts)
                                    .map(|dt| dt.timestamp())
                                    .unwrap_or(0)
                            });
                            date_b.cmp(&date_a)
                        });

                        // Keep fewer logs in memory for better performance
                        if cache.logs.len() > cache.filter_criteria.limit * 3 / 2 {
                            cache.logs = cache
                                .logs
                                .iter()
                                .take(cache.filter_criteria.limit)
                                .cloned()
                                .collect();
                        }

                        // Apply filters for the UI
                        let filtered_logs =
                            cache
                                .logs
                                .iter()
                                .filter(|log| {
                                    (cache.filter_criteria.action.is_empty()
                                        || log
                                            .action
                                            .as_ref()
                                            .is_some_and(|a| a == &cache.filter_criteria.action))
                                        && (cache.filter_criteria.interface.is_empty()
                                            || log.interface.as_ref().is_some_and(|i| {
                                                i == &cache.filter_criteria.interface
                                            }))
                                        && (cache.filter_criteria.direction.is_empty()
                                            || log.dir.as_ref().is_some_and(|d| {
                                                d == &cache.filter_criteria.direction
                                            }))
                                })
                                .take(cache.filter_criteria.limit)
                                .cloned()
                                .collect::<Vec<_>>();

                        // Send the filtered logs to the frontend
                        if let Err(e) = window_clone.emit("firewall-logs-updated", filtered_logs) {
                            log::error!("Failed to emit firewall-logs-updated event: {}", e);
                            // Check if window is gone, if so, stop polling
                            if e.to_string().contains("not available") {
                                log::info!("Window no longer available, stopping log polling");
                                break;
                            }
                        }
                    } else {
                        // No new logs, increase backoff counter
                        consecutive_empty_responses += 1;

                        // Gradually increase polling interval up to the maximum
                        if consecutive_empty_responses > 5 {
                            // Exponential backoff with capping
                            poll_interval_ms = (poll_interval_ms * 5 / 4).min(max_poll_interval_ms);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to fetch firewall logs: {}", e);

                    // Check if active listeners is still > 0
                    let still_active = {
                        let cache = log_cache_clone.lock().unwrap();
                        cache.active_listeners > 0
                    };

                    if !still_active {
                        log::info!("No active listeners after error, stopping polling");
                        break;
                    }

                    // On error, wait a bit longer before retrying
                    poll_interval_ms = max_poll_interval_ms;
                }
            }

            // Dynamic polling interval
            tokio::time::sleep(Duration::from_millis(poll_interval_ms)).await;

            // Check again for stop signal after sleep
            {
                let cache = log_cache_clone.lock().unwrap();
                if cache.active_listeners == 0 {
                    log::info!(
                        "No active listeners after sleep, stopping polling for window {}",
                        window_label
                    );
                    break;
                }
            }
        }

        log::info!("Log polling task completed for window {}", window_label);
    });

    Ok(())
}

#[tauri::command]
pub fn stop_log_polling(log_cache: State<'_, Arc<Mutex<LogCache>>>) -> Result<(), String> {
    // Acquire the lock and decrement active_listeners
    let mut cache = log_cache.lock().unwrap();

    // Log the current state before changing it
    log::info!(
        "Stopping log polling, current active listeners: {}",
        cache.active_listeners
    );

    if cache.active_listeners > 0 {
        cache.active_listeners -= 1;
        log::info!("Decreased active listeners to: {}", cache.active_listeners);
    } else {
        log::warn!("stop_log_polling called when active_listeners was already 0");
    }

    // Immediately reset last_digest to ensure we get fresh data when polling resumes
    if cache.active_listeners == 0 {
        log::info!("All listeners stopped, resetting last_digest");
        cache.last_digest = String::new();
    }

    Ok(())
}

#[tauri::command]
pub fn clear_log_cache(log_cache: State<'_, Arc<Mutex<LogCache>>>) -> Result<(), String> {
    let mut cache = log_cache.lock().unwrap();
    cache.logs.clear();
    cache.last_digest = String::new();
    Ok(())
}
