use base64::{engine::general_purpose, Engine as _};
use log::{error, info};
use reqwest::{
    header::{HeaderMap, AUTHORIZATION, HeaderValue, CONTENT_TYPE},
    Client, Response,
};
use serde_json::Value;
use std::cmp::min;
use std::time::Duration;

/// Makes an HTTP request with a JSON payload
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
        let auth_string = format!("{}:{}", key, secret);
        let auth = general_purpose::STANDARD.encode(auth_string.as_bytes());
        request_builder = request_builder.header(AUTHORIZATION, format!("Basic {}", auth));

        info!(
            "Using auth header: Basic {}...{}",
            &auth[..min(6, auth.len())],
            &auth[auth.len().saturating_sub(4)..]
        );
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
                let error_message = match status.as_u16() {
                    401 => "Authentication failed (HTTP 401): Your API key or secret is incorrect".to_string(),
                    403 => "Permission denied (HTTP 403): Your API credentials don't have sufficient permissions".to_string(),
                    404 => {
                        if url.contains("/api/core/tunables/") {
                            "API endpoint not found (HTTP 404): Tunables API requires OPNsense 25.x or newer".to_string()
                        } else {
                            "API endpoint not found (HTTP 404): Check your firewall URL and port".to_string()
                        }
                    },
                    _ => format!("Request to {} failed with status {}: {}", url, status, body)
                };

                error!("{}", error_message);
                Err(error_message)
            }
        }
        Err(e) => {
            let error_message = if e.is_timeout() {
                format!(
                    "Connection timed out: Server at {} is unreachable or not responding. This may be due to high load on the firewall or network congestion.",
                    url
                )
            } else if e.is_connect() {
                // More detailed connection error message
                if e.to_string().contains("proxy") {
                    format!("Proxy connection error: Unable to connect through proxy to {}. Check your proxy settings.", url)
                } else if e.to_string().contains("refused") {
                    format!("Connection refused: The server at {} actively refused the connection. Please verify the port is correct and any firewall rules allow this connection.", url)
                } else if e.to_string().contains("reset") {
                    format!("Connection reset: The connection to {} was reset. This may indicate network instability or an intermediate firewall blocking the connection.", url)
                } else {
                    format!("Connection error: Unable to connect to server at {}. Check your network connectivity, firewall settings, and verify the server is running.", url)
                }
            } else if e.is_status() {
                format!(
                    "Invalid status: The server at {} returned an unexpected response. This may indicate API changes or incompatibility.",
                    url
                )
            } else if e.to_string().contains("dns error") || e.to_string().contains("not resolve") {
                format!(
                    "DNS resolution error: Could not resolve hostname in URL {}. Please check your DNS settings and verify the hostname is correct.",
                    url
                )
            } else if e.to_string().contains("certificate")
                || e.to_string().contains("SSL")
                || e.to_string().contains("TLS")
            {
                format!("SSL/TLS error: There was a problem with the server's security certificate at {}. This is expected for self-signed certificates and doesn't affect functionality.", url)
            } else if e.to_string().contains("handshake") {
                format!("TLS handshake error: Failed to establish secure connection to {}. This may be due to protocol incompatibility or firewall restrictions.", url)
            } else {
                format!("Request to {} failed: {} - Please check your network connectivity and firewall configuration.", url, e)
            };

            error!("{}", error_message);
            Err(error_message)
        }
    }
}

/// Makes an HTTP request with form data
/// This is used for endpoints that expect application/x-www-form-urlencoded content
/// instead of JSON
pub async fn make_http_request_with_form_data(
    request_type: &str,
    url: &str,
    form_data: String,
    headers: Option<HeaderMap>,
    timeout_seconds: Option<u64>,
    api_key: Option<&str>,
    api_secret: Option<&str>,
) -> Result<Response, String> {
    info!("Making a {} form data request to {}", request_type, url);

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
        let auth_string = format!("{}:{}", key, secret);
        let auth = general_purpose::STANDARD.encode(auth_string.as_bytes());
        request_builder = request_builder.header(AUTHORIZATION, format!("Basic {}", auth));

        info!(
            "Using auth header: Basic {}...{}",
            &auth[..min(6, auth.len())],
            &auth[auth.len().saturating_sub(4)..]
        );
    }

    // Set the Content-Type header for form data
    let mut request_headers = headers.unwrap_or_default();
    request_headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"),
    );
    request_builder = request_builder.headers(request_headers);

    // Set the form data as a raw string in the request body
    request_builder = request_builder.body(form_data);

    info!("Form data request build is finalized: {:?}", &request_builder);

    match request_builder.send().await {
        Ok(response) => {
            if response.status().is_success() {
                info!("Request to {} successful", url);
                Ok(response)
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_else(|_| "".to_string());
                let error_message = match status.as_u16() {
                    401 => "Authentication failed (HTTP 401): Your API key or secret is incorrect".to_string(),
                    403 => "Permission denied (HTTP 403): Your API credentials don't have sufficient permissions".to_string(),
                    404 => {
                        if url.contains("/api/core/tunables/") {
                            "API endpoint not found (HTTP 404): Tunables API requires OPNsense 25.x or newer".to_string()
                        } else {
                            "API endpoint not found (HTTP 404): Check your firewall URL and port".to_string()
                        }
                    },
                    _ => format!("Request to {} failed with status {}: {}", url, status, body)
                };

                error!("{}", error_message);
                Err(error_message)
            }
        }
        Err(e) => {
            let error_message = if e.is_timeout() {
                format!(
                    "Connection timed out: Server at {} is unreachable or not responding. This may be due to high load on the firewall or network congestion.",
                    url
                )
            } else if e.is_connect() {
                // More detailed connection error message
                if e.to_string().contains("proxy") {
                    format!("Proxy connection error: Unable to connect through proxy to {}. Check your proxy settings.", url)
                } else if e.to_string().contains("refused") {
                    format!("Connection refused: The server at {} actively refused the connection. Please verify the port is correct and any firewall rules allow this connection.", url)
                } else if e.to_string().contains("reset") {
                    format!("Connection reset: The connection to {} was reset. This may indicate network instability or an intermediate firewall blocking the connection.", url)
                } else {
                    format!("Connection error: Unable to connect to server at {}. Check your network connectivity, firewall settings, and verify the server is running.", url)
                }
            } else if e.is_status() {
                format!(
                    "Invalid status: The server at {} returned an unexpected response. This may indicate API changes or incompatibility.",
                    url
                )
            } else if e.to_string().contains("dns error") || e.to_string().contains("not resolve") {
                format!(
                    "DNS resolution error: Could not resolve hostname in URL {}. Please check your DNS settings and verify the hostname is correct.",
                    url
                )
            } else if e.to_string().contains("certificate")
                || e.to_string().contains("SSL")
                || e.to_string().contains("TLS")
            {
                format!("SSL/TLS error: There was a problem with the server's security certificate at {}. This is expected for self-signed certificates and doesn't affect functionality.", url)
            } else if e.to_string().contains("handshake") {
                format!("TLS handshake error: Failed to establish secure connection to {}. This may be due to protocol incompatibility or firewall restrictions.", url)
            } else {
                format!("Request to {} failed: {} - Please check your network connectivity and firewall configuration.", url, e)
            };

            error!("{}", error_message);
            Err(error_message)
        }
    }
}
