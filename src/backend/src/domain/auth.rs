use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::api::KeyValueEntry;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthDefinitions {
    pub environments: HashMap<String, AuthEnvironment>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum AuthEnvironment {
    Fixed {
        credentials: FixedAuthCredentials,
    },
    Http {
        request: AuthHttpRequest,
        response: AuthResponse,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FixedAuthCredentials {
    pub source: String,
    #[serde(default)]
    pub mappings: Vec<FixedAuthMapping>,
    pub default: Option<FixedAuthDefault>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FixedAuthMapping {
    pub id: String,
    pub password: String,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FixedAuthDefault {
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthHttpRequest {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<KeyValueEntry>,
    pub body: AuthBody,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthResponse {
    #[serde(default)]
    pub inject: Vec<ResponseInjectRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResponseInjectRule {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthBody {
    Json { text: String },
    Form { form: Vec<KeyValueEntry> },
}
