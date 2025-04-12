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
            logs: Vec::with_capacity(1000),
            last_digest: String::new(),
            last_update: Instant::now(),
            active_listeners: 0,
            filter_criteria: LogFilterCriteria {
                action: String::new(),
                interface: String::new(),
                direction: String::new(),
                limit: 1000,
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
        "{}:{}/api/diagnostics/firewall/log/?digest={}&limit=1000",
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
        if let Some(log) = new_logs.first() {
            if let Some(digest) = &log.digest {
                cache.last_digest = digest.clone();
            }
        }
        if cache.logs.len() > cache.filter_criteria.limit * 2 {
            cache.logs = cache
                .logs
                .iter()
                .take(cache.filter_criteria.limit * 2)
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

    let log_cache_clone = log_cache.inner().clone();
    let window_clone = window.clone();

    tauri::async_runtime::spawn(async move {
        let database = window_clone.state::<Database>();

        loop {
            let digest;
            {
                let cache = log_cache_clone.lock().unwrap();
                if cache.active_listeners == 0 {
                    break;
                }
                digest = cache.last_digest.clone();
            }
            match fetch_firewall_logs(database.clone(), &digest).await {
                Ok(new_logs) => {
                    if !new_logs.is_empty() {
                        let mut cache = log_cache_clone.lock().unwrap();

                        for log in &new_logs {
                            cache.logs.push(log.clone());
                        }

                        if let Some(log) = new_logs.first() {
                            if let Some(digest) = &log.digest {
                                cache.last_digest = digest.clone();
                            }
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

                        if cache.logs.len() > cache.filter_criteria.limit * 2 {
                            cache.logs = cache
                                .logs
                                .iter()
                                .take(cache.filter_criteria.limit * 2)
                                .cloned()
                                .collect();
                        }

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

                        let _ = window_clone.emit("firewall-logs-updated", filtered_logs);
                    }
                }
                Err(e) => {
                    error!("Failed to fetch firewall logs: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    Ok(())
}

#[tauri::command]
pub fn stop_log_polling(log_cache: State<'_, Arc<Mutex<LogCache>>>) -> Result<(), String> {
    let mut cache = log_cache.lock().unwrap();
    if cache.active_listeners > 0 {
        cache.active_listeners -= 1;
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
