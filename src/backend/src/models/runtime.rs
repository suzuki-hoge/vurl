use std::collections::HashMap;

use crate::domain::environment::EnvironmentDefinition;

#[derive(Debug, Clone)]
pub struct RuntimeEnvironmentState {
    pub constants: HashMap<String, String>,
    pub variables: HashMap<String, String>,
    pub masks: HashMap<String, String>,
}

impl From<&EnvironmentDefinition> for RuntimeEnvironmentState {
    fn from(value: &EnvironmentDefinition) -> Self {
        let constants = value
            .constants
            .iter()
            .map(|(key, item)| (key.clone(), item.value.clone()))
            .collect();
        let variables = value
            .variables
            .iter()
            .map(|(key, item)| (key.clone(), item.value.clone()))
            .collect();
        let masks = value
            .variables
            .iter()
            .filter_map(|(key, item)| item.mask.as_ref().map(|mask| (key.clone(), mask.clone())))
            .collect();

        Self {
            constants,
            variables,
            masks,
        }
    }
}
