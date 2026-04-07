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
    Form { form: Vec<KeyValueEntry> },
}
