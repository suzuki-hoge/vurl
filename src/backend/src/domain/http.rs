use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct KeyValueEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RequestBodyDraft {
    Json { text: String },
    Form { form: Vec<KeyValueEntry> },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuthInputMode {
    Preset,
    Manual,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct AuthCredentials {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HeaderEntry {
    pub key: String,
    pub value: String,
}
