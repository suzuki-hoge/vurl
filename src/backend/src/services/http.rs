use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::{Method, RequestBuilder};
use std::time::Duration;

use crate::domain::http::{HeaderEntry, KeyValueEntry, RequestBodyDraft};

#[derive(Debug, Clone)]
pub struct PreparedRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<KeyValueEntry>,
    pub body: RequestBodyDraft,
}

#[derive(Debug, Clone)]
pub struct ExecutedRequest {
    pub status: u16,
    pub headers: Vec<HeaderEntry>,
    pub content_type: Option<String>,
    pub body_text: String,
    pub body_base64: Option<String>,
}

pub async fn send(prepared: &PreparedRequest, timeout_ms: u64) -> Result<ExecutedRequest> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()?;
    let method = Method::from_bytes(prepared.method.as_bytes())?;
    let mut req = client.request(method, &prepared.url);
    for header in &prepared.headers {
        req = req.header(&header.key, &header.value);
    }
    req = apply_body(req, &prepared.body)?;

    let response = req.send().await?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let headers = response_headers(response.headers());
    let body_bytes = response.bytes().await?;
    let body_text = String::from_utf8_lossy(&body_bytes).into_owned();
    let body_base64 = content_type
        .as_deref()
        .filter(|value| is_image_content_type(value))
        .map(|_| STANDARD.encode(&body_bytes));

    Ok(ExecutedRequest {
        status: status.as_u16(),
        headers,
        content_type,
        body_text,
        body_base64,
    })
}

pub fn build_url(base_url: &str, url_path: &str, query: &[KeyValueEntry]) -> String {
    let mut url = format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        url_path.trim_start_matches('/')
    );

    if !query.is_empty() {
        let params = query
            .iter()
            .map(|entry| format!("{}={}", encode(&entry.key), encode(&entry.value)))
            .collect::<Vec<_>>()
            .join("&");
        url.push('?');
        url.push_str(&params);
    }

    url
}

pub fn build_curl(prepared: &PreparedRequest) -> String {
    let mut lines = vec![format!("curl -X {} '{}'", prepared.method, prepared.url)];
    for header in &prepared.headers {
        lines.push(format!(
            "  -H '{}: {}'",
            escape_single(&header.key),
            escape_single(&header.value)
        ));
    }

    match &prepared.body {
        RequestBodyDraft::Json { text } => {
            if !text.trim().is_empty() {
                lines.push(format!("  --data-raw '{}'", escape_single(text)));
            }
        }
        RequestBodyDraft::Form { form } => {
            for entry in form {
                lines.push(format!(
                    "  -d '{}={}'",
                    escape_single(&entry.key),
                    escape_single(&entry.value)
                ));
            }
        }
    }

    lines.join(" \\\n")
}

fn apply_body(req: RequestBuilder, body: &RequestBodyDraft) -> Result<RequestBuilder> {
    Ok(match body {
        RequestBodyDraft::Json { text } => {
            if text.trim().is_empty() {
                req
            } else {
                req.body(text.clone())
            }
        }
        RequestBodyDraft::Form { form } => {
            let pairs: Vec<(String, String)> = form
                .iter()
                .map(|entry| (entry.key.clone(), entry.value.clone()))
                .collect();
            req.form(&pairs)
        }
    })
}

fn response_headers(headers: &reqwest::header::HeaderMap) -> Vec<HeaderEntry> {
    headers
        .iter()
        .map(|(key, value)| HeaderEntry {
            key: key.to_string(),
            value: value.to_str().unwrap_or_default().to_string(),
        })
        .collect()
}

fn is_image_content_type(content_type: &str) -> bool {
    content_type
        .split(';')
        .next()
        .is_some_and(|value| value.trim().starts_with("image/"))
}

fn encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn escape_single(value: &str) -> String {
    value.replace('\'', "'\\''")
}
