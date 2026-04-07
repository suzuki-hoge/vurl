use anyhow::{Context, Result};

use crate::{
    domain::{
        auth::AuthEnvironment,
        http::{AuthCredentials, AuthInputMode, HeaderEntry, KeyValueEntry, RequestBodyDraft},
    },
    runtime::store::RuntimeStore,
    services::{
        auth::{authenticate, resolve_auth_credentials},
        http::{PreparedRequest, build_curl, build_url, send},
        logging::append_request_log,
        resolver::ResolveContext,
    },
};

pub const REQUEST_TIMEOUT_MS: u64 = 3_000;

#[derive(Debug, Clone)]
pub struct ExecuteRequestInput {
    pub project: String,
    pub environment: String,
    pub path: String,
    pub method: String,
    pub url_path: String,
    pub query: Vec<KeyValueEntry>,
    pub headers: Vec<KeyValueEntry>,
    pub body: RequestBodyDraft,
    pub auth: RequestAuth,
}

#[derive(Debug, Clone)]
pub struct RequestAuth {
    pub enabled: bool,
    pub input_mode: AuthInputMode,
    pub preset_name: Option<String>,
    pub credentials: AuthCredentials,
}

#[derive(Debug, Clone)]
pub struct ExecuteRequestResult {
    pub status: u16,
    pub headers: Vec<HeaderEntry>,
    pub content_type: Option<String>,
    pub body: String,
    pub body_base64: Option<String>,
    pub retried_auth: bool,
    pub notifications: Vec<ResponseNotification>,
    pub current_log_file: String,
}

#[derive(Debug, Clone)]
pub struct ResponseNotification {
    pub code: ResponseNotificationCode,
    pub kind: ResponseNotificationKind,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseNotificationCode {
    Authenticated,
    Timeout,
    Generic,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseNotificationKind {
    Info,
    Error,
}

pub async fn execute_request(
    store: &RuntimeStore,
    input: ExecuteRequestInput,
) -> Result<ExecuteRequestResult> {
    let mut env_state = store.env_state(&input.project, &input.environment)?;
    let auth_environment = store
        .project(&input.project)?
        .auth
        .environments
        .get(&input.environment)
        .with_context(|| {
            format!(
                "auth environment not found: {}/{}",
                input.project, input.environment
            )
        })?;
    let auth = resolve_auth_credentials(
        store,
        &input.project,
        &input.environment,
        &input.auth.input_mode,
        input.auth.preset_name.as_deref(),
        &input.auth.credentials,
    )?;
    let mut http_authenticated = false;

    if input.auth.enabled
        && should_authenticate_before_request(
            store,
            &input.project,
            &input.environment,
            &env_state,
        )?
    {
        let updates = authenticate(store, &input.project, &input.environment, &auth).await?;
        env_state = store.update_env_variables(&input.project, &input.environment, &updates)?;
        http_authenticated = matches!(auth_environment, AuthEnvironment::Http { .. });
    }

    let mut attempt = 0;
    let mut retried_auth = false;

    loop {
        let (resolver, prepared) = prepare_request(&input, env_state.clone(), auth.clone())?;
        let curl = build_curl(&prepared);
        let response = send(&prepared, REQUEST_TIMEOUT_MS).await?;

        if input.auth.enabled && attempt == 0 && matches!(response.status, 401 | 403) {
            let updates = authenticate(store, &input.project, &input.environment, &auth).await?;
            env_state = store.update_env_variables(&input.project, &input.environment, &updates)?;
            retried_auth = true;
            if matches!(auth_environment, AuthEnvironment::Http { .. }) {
                http_authenticated = true;
            }
            attempt += 1;
            continue;
        }

        let log_file =
            append_request_log(store, &input.project, &resolver, &curl, &response.body_text)?;
        let notifications = if http_authenticated {
            vec![ResponseNotification {
                code: ResponseNotificationCode::Authenticated,
                kind: ResponseNotificationKind::Info,
                message: "自動認証を実行しました".to_string(),
            }]
        } else {
            Vec::new()
        };
        return Ok(ExecuteRequestResult {
            status: response.status,
            headers: response.headers,
            content_type: response.content_type,
            body: response.body_text,
            body_base64: response.body_base64,
            retried_auth,
            notifications,
            current_log_file: log_file.display().to_string(),
        });
    }
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

fn prepare_request(
    input: &ExecuteRequestInput,
    environment: crate::models::runtime::RuntimeEnvironmentState,
    auth: AuthCredentials,
) -> Result<(ResolveContext, PreparedRequest)> {
    let resolver = ResolveContext { environment, auth };
    let url_path = resolver.resolve_string(&input.url_path)?;
    let query = resolver.resolve_entries(&input.query)?;
    let headers = resolver.resolve_entries(&input.headers)?;
    let body = resolver.resolve_body(&input.body)?;
    let base_url = resolver
        .environment
        .constants
        .get("base_url")
        .cloned()
        .context("base_url constant not found")?;
    let url = build_url(&base_url, &url_path, &query);

    Ok((
        resolver,
        PreparedRequest {
            method: input.method.clone(),
            url,
            headers,
            body,
        },
    ))
}
