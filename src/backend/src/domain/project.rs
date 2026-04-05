use std::collections::HashMap;

use serde::Serialize;

use crate::domain::{
    auth::AuthDefinitions, environment::EnvironmentDefinition, request::RequestDefinition,
};

#[derive(Debug, Clone)]
pub struct ProjectData {
    pub name: String,
    pub requests: HashMap<String, RequestDefinition>,
    pub environments: HashMap<String, EnvironmentDefinition>,
    pub auth: AuthDefinitions,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RequestTreeNode {
    Directory {
        name: String,
        path: String,
        children: Vec<RequestTreeNode>,
    },
    Request {
        name: String,
        path: String,
        title: String,
        method: String,
    },
}
