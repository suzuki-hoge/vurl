use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use reqwest::Method;
use serde_json::Value;

use crate::{
    domain::{
        api::{AuthCredentials, AuthInputMode, HeaderEntry, RequestBodyDraft},
        auth::{AuthBody, AuthCredentialPreset, AuthEnvironment, ResponseInjectRule},
    },
    runtime::store::RuntimeStore,
    services::{
        logging::append_raw_log, request_execution::REQUEST_TIMEOUT_MS, resolver::ResolveContext,
    },
};

pub fn resolve_auth_credentials(
    store: &RuntimeStore,
    project: &str,
    environment: &str,
    input_mode: &AuthInputMode,
    preset_name: Option<&str>,
    manual_credentials: &AuthCredentials,
) -> Result<AuthCredentials> {
    match input_mode {
        AuthInputMode::Manual => Ok(manual_credentials.clone()),
        AuthInputMode::Preset => {
            let auth_definitions = &store.project(project)?.auth;
            let auth = auth_definitions
                .environments
                .get(environment)
                .with_context(|| format!("auth environment not found: {project}/{environment}"))?;

            let preset_name = preset_name.context("auth preset name is required")?;
            let preset = match auth {
                AuthEnvironment::Fixed { credentials, .. } => {
                    find_preset(&credentials.presets, preset_name)?
                }
                AuthEnvironment::Http { credentials, .. } => {
                    let credentials = credentials
                        .as_ref()
                        .context("auth presets not configured for http auth")?;
                    find_preset(&credentials.presets, preset_name)?
                }
            };

            Ok(AuthCredentials {
                id: preset.id.clone(),
                password: preset.password.clone().unwrap_or_default(),
            })
        }
    }
}

pub async fn authenticate(
    store: &RuntimeStore,
    project: &str,
    environment: &str,
    credentials: &AuthCredentials,
) -> Result<HashMap<String, String>> {
    let auth_definitions = &store.project(project)?.auth;
    let auth = auth_definitions
        .environments
        .get(environment)
        .with_context(|| format!("auth environment not found: {project}/{environment}"))?;

    match auth {
        AuthEnvironment::Fixed { mappings, .. } => {
            for mapping in &mappings.items {
                if mapping.id == credentials.id {
                    return Ok(mapping.variables.clone());
                }
            }

            if let Some(default) = &mappings.default {
                return Ok(default.variables.clone());
            }

            bail!("authentication mapping not found")
        }
        AuthEnvironment::Http {
            request,
            response: auth_response,
            ..
        } => {
            let runtime = store.env_state(project, environment)?;
            let resolver = ResolveContext {
                environment: runtime,
                auth: credentials.clone(),
            };

            let method = Method::from_bytes(request.method.as_bytes())
                .with_context(|| format!("invalid auth method: {}", request.method))?;
            let url = resolver.resolve_string(&request.url)?;
            let headers = resolver.resolve_entries(&request.headers)?;
            let body = resolve_auth_body(&resolver, &request.body)?;
            let curl = build_auth_curl(&request.method, &url, &headers, &body);

            let client = reqwest::Client::builder()
                .timeout(Duration::from_millis(REQUEST_TIMEOUT_MS))
                .build()?;
            let mut req = client.request(method, url);
            for header in headers {
                req = req.header(&header.key, &header.value);
            }
            req = apply_request_body(req, &body)?;

            let response = req.send().await?;
            let status = response.status();
            let text = response.text().await?;
            let _ = append_raw_log(store, project, &curl, &text);
            if !status.is_success() {
                bail!("auth request failed: {status} {text}");
            }

            let json: Value = serde_json::from_str(&text).unwrap_or(Value::String(text));
            let mut updates = HashMap::new();
            for rule in &auth_response.inject {
                let value = extract_json_path(&json, rule)?
                    .with_context(|| format!("auth response path not found: {}", rule.from))?;
                updates.insert(rule.to.clone(), value);
            }
            Ok(updates)
        }
    }
}

fn build_auth_curl(
    method: &str,
    url: &str,
    headers: &[crate::domain::api::KeyValueEntry],
    body: &RequestBodyDraft,
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

fn find_preset<'a>(
    presets: &'a [AuthCredentialPreset],
    preset_name: &str,
) -> Result<&'a AuthCredentialPreset> {
    presets
        .iter()
        .find(|preset| preset.name == preset_name)
        .with_context(|| format!("auth preset not found: {preset_name}"))
}

fn resolve_auth_body(resolver: &ResolveContext, body: &AuthBody) -> Result<RequestBodyDraft> {
    match body {
        AuthBody::Json { text } => Ok(RequestBodyDraft::Json {
            text: resolver.resolve_string(text)?,
        }),
        AuthBody::Form { form } => Ok(RequestBodyDraft::Form {
            form: resolver.resolve_entries(form)?,
        }),
    }
}

fn apply_request_body(
    req: reqwest::RequestBuilder,
    body: &RequestBodyDraft,
) -> Result<reqwest::RequestBuilder> {
    Ok(match body {
        RequestBodyDraft::Json { text } => {
            if text.trim().is_empty() {
                req
            } else if let Ok(json) = serde_json::from_str::<Value>(text) {
                req.json(&json)
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

fn extract_json_path(json: &Value, rule: &ResponseInjectRule) -> Result<Option<String>> {
    if !rule.from.starts_with("$.") {
        bail!("unsupported json path: {}", rule.from);
    }

    let mut current = json;
    for segment in rule.from.trim_start_matches("$.").split('.') {
        current = match current {
            Value::Object(map) => match map.get(segment) {
                Some(value) => value,
                None => return Ok(None),
            },
            _ => return Ok(None),
        };
    }

    let result = match current {
        Value::Null => None,
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::String(value) => Some(value.clone()),
        other => Some(other.to_string()),
    };

    Ok(result)
}

fn escape_single(value: &str) -> String {
    value.replace('\'', "'\\''")
}

pub fn response_headers(headers: &reqwest::header::HeaderMap) -> Vec<HeaderEntry> {
    headers
        .iter()
        .map(|(key, value)| HeaderEntry {
            key: key.to_string(),
            value: value.to_str().unwrap_or_default().to_string(),
        })
        .collect()
}
