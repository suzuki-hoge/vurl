use serde::{Deserialize, Serialize};

use crate::domain::http::KeyValueEntry;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestDefinition {
    pub name: String,
    pub method: String,
    pub path: String,
    pub auth: bool,
    pub request: RequestPayload,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestPayload {
    #[serde(default)]
    pub query: Vec<KeyValueEntry>,
    #[serde(default)]
    pub headers: Vec<KeyValueEntry>,
    pub body: RequestBodyDefinition,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RequestBodyDefinition {
    Json { text: String },
    Form { form: Vec<FormFieldDefinition> },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormFieldDefinition {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub items: Vec<FormFieldSelectItemDefinition>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormFieldSelectItemDefinition {
    pub value: String,
    pub description: String,
    #[serde(default)]
    pub default: bool,
}

fn default_enabled() -> bool {
    true
}
