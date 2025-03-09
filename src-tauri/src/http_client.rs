use base64::{engine::general_purpose, Engine as _};
use log::{error, info};
use reqwest::{
    header::{HeaderMap, AUTHORIZATION},
    Client, Response,
};
use serde_json::Value;
use std::time::Duration;

pub async fn make_http_request(
    request_type: &str,
    url: &str,
    payload: Option<Value>,
    headers: Option<HeaderMap>,
    timeout_seconds: Option<u64>,
    api_key: Option<&str>,
    api_secret: Option<&str>,
) -> Result<Response, String> {
    info!("Making a {} request to {}", request_type, url);

    let client_builder = Client::builder().danger_accept_invalid_certs(true);
    let client = if let Some(timeout_sec) = timeout_seconds {
        client_builder
            .timeout(Duration::from_secs(timeout_sec))
            .build()
    } else {
        client_builder.build()
    }
    .map_err(|e| {
        let error_message = format!("Failed to build HTTP client: {}", e);
        error!("{}", error_message);
        error_message
    })?;

    let mut request_builder = match request_type {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PATCH" => client.patch(url),
        "PUT" => client.put(url),
        _ => {
            let error_message = "Invalid request type".to_string();
            error!("{}", error_message);
            return Err(error_message);
        }
    };

    if let (Some(key), Some(secret)) = (api_key, api_secret) {
        let auth = general_purpose::STANDARD.encode(format!("{}:{}", key, secret));
        request_builder = request_builder.header(AUTHORIZATION, format!("Basic {}", auth));
    }

    if let Some(headers) = headers {
        request_builder = request_builder.headers(headers);
    }

    if let Some(payload) = payload {
        request_builder = request_builder.json(&payload);
    }

    info!("Request build is finalized: {:?}", &request_builder);

    match request_builder.send().await {
        Ok(response) => {
            if response.status().is_success() {
                info!("Request to {} successful", url);
                Ok(response)
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_else(|_| "".to_string());
                let error_message =
                    format!("Request to {} failed with status {}: {}", url, status, body);
                error!("{}", error_message);
                Err(error_message)
            }
        }
        Err(e) => {
            let error_message = format!("Request to {} failed: {}", url, e);
            error!("{}", error_message);
            Err(error_message)
        }
    }
}
