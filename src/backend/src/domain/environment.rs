use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvironmentDefinition {
    pub name: String,
    pub order: Option<u32>,
    #[serde(default)]
    pub constants: HashMap<String, EnvironmentValue>,
    #[serde(default)]
    pub variables: HashMap<String, EnvironmentValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvironmentValue {
    pub value: String,
    pub mask: Option<String>,
}
