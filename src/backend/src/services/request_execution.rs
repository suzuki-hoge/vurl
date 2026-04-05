use anyhow::{Context, Result};
use reqwest::Method;

use crate::{
    domain::api::{SendRequest, SendResponse},
    runtime::store::RuntimeStore,
    services::{
        auth::{authenticate, response_headers},
        logging::append_request_log,
        resolver::ResolveContext,
    },
};

pub async fn execute_request(store: &RuntimeStore, payload: SendRequest) -> Result<SendResponse> {
    let mut env_state = store.env_state(&payload.project, &payload.environment)?;
    let auth = payload.auth_credentials.clone();

    if payload.auth_enabled {
        let updates = authenticate(store, &payload.project, &payload.environment, &auth).await?;
        env_state = store.update_env_variables(&payload.project, &payload.environment, &updates)?;
    }

    let mut attempt = 0;
    let mut retried_auth = false;

    loop {
        let resolver = ResolveContext {
            environment: env_state.clone(),
            auth: auth.clone(),
        };
        let method = Method::from_bytes(payload.method.as_bytes())
            .with_context(|| format!("invalid method: {}", payload.method))?;
        let url_path = resolver.resolve_string(&payload.url_path)?;
        let query = resolver.resolve_entries(&payload.query)?;
        let headers = resolver.resolve_entries(&payload.headers)?;
        let body = resolver.resolve_body(&payload.body)?;
        let base_url = resolver
            .environment
            .constants
            .get("base_url")
            .cloned()
            .context("base_url constant not found")?;
        let full_url = build_url(&base_url, &url_path, &query);

        let client = reqwest::Client::builder().build()?;
        let mut req = client.request(method.clone(), &full_url);
        for header in &headers {
            req = req.header(&header.key, &header.value);
        }

        let curl = build_curl(&payload.method, &full_url, &headers, &body);

        req = apply_body(req, &body)?;
        let response = req.send().await?;
        let status = response.status();
        let headers_out = response_headers(response.headers());
        let response_text = response.text().await?;

        if payload.auth_enabled && attempt == 0 && matches!(status.as_u16(), 401 | 403) {
            let updates =
                authenticate(store, &payload.project, &payload.environment, &auth).await?;
            env_state =
                store.update_env_variables(&payload.project, &payload.environment, &updates)?;
            retried_auth = true;
            attempt += 1;
            continue;
        }

        let log_file =
            append_request_log(store, &payload.project, &resolver, &curl, &response_text)?;
        return Ok(SendResponse {
            status: status.as_u16(),
            headers: headers_out,
            body: response_text,
            retried_auth,
            current_log_file: log_file.display().to_string(),
        });
    }
}

fn build_url(
    base_url: &str,
    url_path: &str,
    query: &[crate::domain::api::KeyValueEntry],
) -> String {
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

fn build_curl(
    method: &str,
    url: &str,
    headers: &[crate::domain::api::KeyValueEntry],
    body: &crate::domain::api::RequestBodyDraft,
) -> String {
    let mut lines = vec![format!("curl -X {method} '{url}'")];
    for header in headers {
        lines.push(format!(
            "  -H '{}: {}'",
            escape_single(&header.key),
            escape_single(&header.value)
        ));
    }

    match body {
        crate::domain::api::RequestBodyDraft::Json { text } => {
            if !text.trim().is_empty() {
                lines.push(format!("  --data-raw '{}'", escape_single(text)));
            }
        }
        crate::domain::api::RequestBodyDraft::Form { form } => {
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

fn apply_body(
    req: reqwest::RequestBuilder,
    body: &crate::domain::api::RequestBodyDraft,
) -> Result<reqwest::RequestBuilder> {
    Ok(match body {
        crate::domain::api::RequestBodyDraft::Json { text } => {
            if text.trim().is_empty() {
                req
            } else {
                req.body(text.clone())
            }
        }
        crate::domain::api::RequestBodyDraft::Form { form } => {
            let pairs: Vec<(String, String)> = form
                .iter()
                .map(|entry| (entry.key.clone(), entry.value.clone()))
                .collect();
            req.form(&pairs)
        }
    })
}

fn encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn escape_single(value: &str) -> String {
    value.replace('\'', "'\\''")
}
