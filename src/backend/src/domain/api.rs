use serde::{Deserialize, Serialize};

use crate::domain::project::RequestTreeNode;
use crate::domain::request::RequestDefinition;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectSummary {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentSummary {
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

#[derive(Debug, Clone, Serialize)]
pub struct SendResponse {
    pub status: u16,
    pub headers: Vec<HeaderEntry>,
    pub body: String,
    pub retried_auth: bool,
    pub current_log_file: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeaderEntry {
    pub key: String,
    pub value: String,
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
    #[serde(default)]
    pub auth_credentials: AuthCredentials,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AuthCredentials {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyValueEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RequestBodyDraft {
    Json { text: String },
    Form { form: Vec<KeyValueEntry> },
}
