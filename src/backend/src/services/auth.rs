use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use reqwest::Method;
use serde_json::Value;

use crate::{
    domain::{
        api::{AuthCredentials, HeaderEntry, RequestBodyDraft},
        auth::{AuthBody, AuthEnvironment, ResponseInjectRule},
    },
    runtime::store::RuntimeStore,
    services::resolver::ResolveContext,
};

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
        AuthEnvironment::Fixed { credentials: fixed } => {
            for mapping in &fixed.mappings {
                if mapping.id == credentials.id && mapping.password == credentials.password {
                    return Ok(mapping.variables.clone());
                }
            }

            if let Some(default) = &fixed.default {
                return Ok(default.variables.clone());
            }

            bail!("authentication mapping not found")
        }
        AuthEnvironment::Http {
            request,
            response: auth_response,
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

            let client = reqwest::Client::builder().build()?;
            let mut req = client.request(method, url);
            for header in headers {
                req = req.header(&header.key, &header.value);
            }
            req = apply_request_body(req, &body)?;

            let response = req.send().await?;
            let status = response.status();
            let text = response.text().await?;
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

pub fn response_headers(headers: &reqwest::header::HeaderMap) -> Vec<HeaderEntry> {
    headers
        .iter()
        .map(|(key, value)| HeaderEntry {
            key: key.to_string(),
            value: value.to_str().unwrap_or_default().to_string(),
        })
        .collect()
}
