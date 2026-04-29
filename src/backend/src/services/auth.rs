use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::{
    domain::{
        auth::{AuthBody, AuthCredentialPreset, AuthEnvironment, ResponseInjectRule},
        http::{AuthCredentials, AuthInputMode, RequestBodyDraft},
    },
    runtime::store::RuntimeStore,
    services::{
        http::{PreparedRequest, build_curl, send},
        logging::append_raw_log,
        request_execution::REQUEST_TIMEOUT_MS,
        resolver::ResolveContext,
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
            request: _,
            response: auth_response,
            ..
        } => {
            let prepared = prepare_http_auth_request(store, project, environment, credentials)?;
            let curl = build_curl(&prepared);
            let response = send(&prepared, REQUEST_TIMEOUT_MS).await?;
            let text = response.body_text;
            let _ = append_raw_log(store, project, &curl, response.status, &text);
            if !(200..300).contains(&response.status) {
                bail!("auth request failed: {} {}", response.status, text);
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

fn prepare_http_auth_request(
    store: &RuntimeStore,
    project: &str,
    environment: &str,
    credentials: &AuthCredentials,
) -> Result<PreparedRequest> {
    let auth_definitions = &store.project(project)?.auth;
    let auth = auth_definitions
        .environments
        .get(environment)
        .with_context(|| format!("auth environment not found: {project}/{environment}"))?;

    let AuthEnvironment::Http { request, .. } = auth else {
        bail!("http auth is not configured")
    };

    let runtime = store.env_state(project, environment)?;
    let resolver = ResolveContext {
        environment: runtime,
        auth: credentials.clone(),
    };

    Ok(PreparedRequest {
        method: request.method.clone(),
        url: resolver.resolve_string(&request.url)?,
        headers: resolver.resolve_entries(&request.headers)?,
        body: resolve_auth_body(&resolver, &request.body)?,
    })
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
