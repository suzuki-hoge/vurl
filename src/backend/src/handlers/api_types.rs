use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        http::{AuthCredentials, AuthInputMode, HeaderEntry, KeyValueEntry, RequestBodyDraft},
        project::RequestTreeNode,
        request::RequestDefinition,
    },
    services::request_execution::{
        ExecuteRequestInput, ExecuteRequestResult, RequestAuth, ResponseNotification,
        ResponseNotificationCode, ResponseNotificationKind,
    },
};

#[derive(Debug, Deserialize)]
pub struct ProjectQuery {
    pub project: String,
}

#[derive(Debug, Deserialize)]
pub struct DefinitionQuery {
    pub project: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectSummary {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentSummary {
    pub name: String,
    #[serde(default)]
    pub auth_presets: Vec<AuthPresetSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthPresetSummary {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeInfo {
    pub root: String,
    pub projects: Vec<ProjectSummary>,
    pub backend_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DefinitionResponse {
    pub path: String,
    pub definition: RequestDefinition,
}

#[derive(Debug, Clone, Serialize)]
pub struct TreeResponse {
    pub project: String,
    pub nodes: Vec<RequestTreeNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogFileResponse {
    pub project: String,
    pub current_log_file: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendRequest {
    pub project: String,
    pub environment: String,
    pub path: String,
    pub method: String,
    pub url_path: String,
    #[serde(default)]
    pub query: Vec<KeyValueEntry>,
    #[serde(default)]
    pub headers: Vec<KeyValueEntry>,
    pub body: RequestBodyDraft,
    pub auth_enabled: bool,
    pub auth_input_mode: AuthInputMode,
    pub auth_preset_name: Option<String>,
    #[serde(default)]
    pub auth_credentials: AuthCredentials,
}

impl From<SendRequest> for ExecuteRequestInput {
    fn from(value: SendRequest) -> Self {
        Self {
            project: value.project,
            environment: value.environment,
            path: value.path,
            method: value.method,
            url_path: value.url_path,
            query: value.query,
            headers: value.headers,
            body: value.body,
            auth: RequestAuth {
                enabled: value.auth_enabled,
                input_mode: value.auth_input_mode,
                preset_name: value.auth_preset_name,
                credentials: value.auth_credentials,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SendResponse {
    pub status: u16,
    pub headers: Vec<HeaderEntry>,
    pub content_type: Option<String>,
    pub body: String,
    pub body_base64: Option<String>,
    pub retried_auth: bool,
    #[serde(default)]
    pub notifications: Vec<ResponseNotificationDto>,
    pub current_log_file: String,
}

impl From<ExecuteRequestResult> for SendResponse {
    fn from(value: ExecuteRequestResult) -> Self {
        Self {
            status: value.status,
            headers: value.headers,
            content_type: value.content_type,
            body: value.body,
            body_base64: value.body_base64,
            retried_auth: value.retried_auth,
            notifications: value.notifications.into_iter().map(Into::into).collect(),
            current_log_file: value.current_log_file,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseNotificationDto {
    pub code: ResponseNotificationCode,
    pub kind: ResponseNotificationKind,
    pub message: String,
}

impl From<ResponseNotification> for ResponseNotificationDto {
    fn from(value: ResponseNotification) -> Self {
        Self {
            code: value.code,
            kind: value.kind,
            message: value.message,
        }
    }
}
