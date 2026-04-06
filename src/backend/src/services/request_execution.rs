use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::Method;
use std::time::Duration;

use crate::{
    domain::{
        api::{
            ResponseNotification, ResponseNotificationCode, ResponseNotificationKind, SendRequest,
            SendResponse,
        },
        auth::AuthEnvironment,
    },
    runtime::store::RuntimeStore,
    services::{
        auth::{authenticate, resolve_auth_credentials, response_headers},
        logging::append_request_log,
        resolver::ResolveContext,
    },
};

pub const REQUEST_TIMEOUT_MS: u64 = 3_000;

pub async fn execute_request(store: &RuntimeStore, payload: SendRequest) -> Result<SendResponse> {
    let mut env_state = store.env_state(&payload.project, &payload.environment)?;
    let auth_environment = store
        .project(&payload.project)?
        .auth
        .environments
        .get(&payload.environment)
        .with_context(|| {
            format!(
                "auth environment not found: {}/{}",
                payload.project, payload.environment
            )
        })?;
    let auth = resolve_auth_credentials(
        store,
        &payload.project,
        &payload.environment,
        &payload.auth_input_mode,
        payload.auth_preset_name.as_deref(),
        &payload.auth_credentials,
    )?;
    let mut http_authenticated = false;

    if payload.auth_enabled
        && should_authenticate_before_request(
            store,
            &payload.project,
            &payload.environment,
            &env_state,
        )?
    {
        let updates = authenticate(store, &payload.project, &payload.environment, &auth).await?;
        env_state = store.update_env_variables(&payload.project, &payload.environment, &updates)?;
        http_authenticated = matches!(auth_environment, AuthEnvironment::Http { .. });
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

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(REQUEST_TIMEOUT_MS))
            .build()?;
        let mut req = client.request(method.clone(), &full_url);
        for header in &headers {
            req = req.header(&header.key, &header.value);
        }

        let curl = build_curl(&payload.method, &full_url, &headers, &body);

        req = apply_body(req, &body)?;
        let response = req.send().await?;
        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
        let headers_out = response_headers(response.headers());
        let response_bytes = response.bytes().await?;
        let response_text = String::from_utf8_lossy(&response_bytes).into_owned();
        let response_body_base64 = content_type
            .as_deref()
            .filter(|value| is_image_content_type(value))
            .map(|_| STANDARD.encode(&response_bytes));

        if payload.auth_enabled && attempt == 0 && matches!(status.as_u16(), 401 | 403) {
            let updates =
                authenticate(store, &payload.project, &payload.environment, &auth).await?;
            env_state =
                store.update_env_variables(&payload.project, &payload.environment, &updates)?;
            retried_auth = true;
            if matches!(auth_environment, AuthEnvironment::Http { .. }) {
                http_authenticated = true;
            }
            attempt += 1;
            continue;
        }

        let log_file =
            append_request_log(store, &payload.project, &resolver, &curl, &response_text)?;
        let notifications = if http_authenticated {
            vec![ResponseNotification {
                code: ResponseNotificationCode::Authenticated,
                kind: ResponseNotificationKind::Info,
                message: "自動認証を実行しました".to_string(),
            }]
        } else {
            Vec::new()
        };
        return Ok(SendResponse {
            status: status.as_u16(),
            headers: headers_out,
            content_type,
            body: response_text,
            body_base64: response_body_base64,
            retried_auth,
            notifications,
            current_log_file: log_file.display().to_string(),
        });
    }
}

fn is_image_content_type(content_type: &str) -> bool {
    content_type
        .split(';')
        .next()
        .is_some_and(|value| value.trim().starts_with("image/"))
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

fn should_authenticate_before_request(
    store: &RuntimeStore,
    project: &str,
    environment: &str,
    env_state: &crate::models::runtime::RuntimeEnvironmentState,
) -> Result<bool> {
    let auth = store
        .project(project)?
        .auth
        .environments
        .get(environment)
        .with_context(|| format!("auth environment not found: {project}/{environment}"))?;

    Ok(match auth {
        AuthEnvironment::Fixed { .. } => true,
        AuthEnvironment::Http { response, .. } => response.inject.iter().any(|rule| {
            env_state
                .variables
                .get(&rule.to)
                .is_none_or(|value| value.trim().is_empty())
        }),
    })
}
