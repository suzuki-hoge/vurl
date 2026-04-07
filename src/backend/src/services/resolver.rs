use std::collections::{HashMap, HashSet};

use anyhow::{Result, bail};

use crate::{
    domain::http::{AuthCredentials, KeyValueEntry, RequestBodyDraft},
    models::runtime::RuntimeEnvironmentState,
};

#[derive(Debug, Clone)]
pub struct ResolveContext {
    pub environment: RuntimeEnvironmentState,
    pub auth: AuthCredentials,
}

impl ResolveContext {
    pub fn resolve_string(&self, input: &str) -> Result<String> {
        let mut output = String::new();
        let mut rest = input;
        let mut missing = HashSet::new();

        while let Some(start) = rest.find("{{") {
            output.push_str(&rest[..start]);
            let after = &rest[start + 2..];
            if let Some(end) = after.find("}}") {
                let key = after[..end].trim();
                if let Some(value) = self.lookup(key) {
                    output.push_str(&value);
                } else {
                    missing.insert(key.to_string());
                }
                rest = &after[end + 2..];
            } else {
                output.push_str(&rest[start..]);
                rest = "";
                break;
            }
        }

        output.push_str(rest);

        if !missing.is_empty() {
            let mut names: Vec<_> = missing.into_iter().collect();
            names.sort();
            bail!("undefined variables: {}", names.join(", "));
        }

        Ok(output)
    }

    pub fn resolve_entries(&self, entries: &[KeyValueEntry]) -> Result<Vec<KeyValueEntry>> {
        entries
            .iter()
            .map(|entry| {
                Ok(KeyValueEntry {
                    key: entry.key.clone(),
                    value: self.resolve_string(&entry.value)?,
                })
            })
            .collect()
    }

    pub fn resolve_body(&self, body: &RequestBodyDraft) -> Result<RequestBodyDraft> {
        match body {
            RequestBodyDraft::Json { text } => Ok(RequestBodyDraft::Json {
                text: self.resolve_string(text)?,
            }),
            RequestBodyDraft::Form { form } => Ok(RequestBodyDraft::Form {
                form: self.resolve_entries(form)?,
            }),
        }
    }

    pub fn masks(&self) -> HashMap<String, String> {
        self.environment
            .masks
            .iter()
            .filter_map(|(key, mask)| {
                self.environment
                    .variables
                    .get(key)
                    .map(|value| (value.clone(), mask.clone()))
            })
            .collect()
    }

    fn lookup(&self, key: &str) -> Option<String> {
        if let Some(value) = self.auth_lookup(key) {
            return Some(value);
        }
        if let Some(value) = self.environment.variables.get(key) {
            return Some(value.clone());
        }
        if let Some(value) = self.environment.constants.get(key) {
            return Some(value.clone());
        }
        None
    }

    fn auth_lookup(&self, key: &str) -> Option<String> {
        match key {
            "auth.id" => Some(self.auth.id.clone()),
            "auth.password" => Some(self.auth.password.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::Result;

    use super::ResolveContext;
    use crate::{
        domain::http::{AuthCredentials, KeyValueEntry, RequestBodyDraft},
        models::runtime::RuntimeEnvironmentState,
    };

    fn context() -> ResolveContext {
        ResolveContext {
            environment: RuntimeEnvironmentState {
                constants: HashMap::from([
                    ("base_url".to_string(), "http://localhost:18080".to_string()),
                    ("tenant_id".to_string(), "tenant-a".to_string()),
                ]),
                variables: HashMap::from([
                    ("auth_token".to_string(), "secret-token".to_string()),
                    ("user_id".to_string(), "42".to_string()),
                ]),
                masks: HashMap::from([("auth_token".to_string(), "xxx".to_string())]),
            },
            auth: AuthCredentials {
                id: "alice".to_string(),
                password: "pw".to_string(),
            },
        }
    }

    #[test]
    fn resolves_known_variables() -> Result<()> {
        let resolved = context().resolve_string("{{base_url}}/users/{{user_id}}")?;
        assert_eq!(resolved, "http://localhost:18080/users/42");
        Ok(())
    }

    #[test]
    fn resolves_form_entries() -> Result<()> {
        let entries = vec![
            KeyValueEntry {
                key: "tenant".to_string(),
                value: "{{tenant_id}}".to_string(),
            },
            KeyValueEntry {
                key: "auth".to_string(),
                value: "{{auth.id}}".to_string(),
            },
        ];

        let resolved = context().resolve_entries(&entries)?;
        assert_eq!(resolved[0].value, "tenant-a");
        assert_eq!(resolved[1].value, "alice");
        Ok(())
    }

    #[test]
    fn returns_error_for_undefined_variable() {
        let error = context()
            .resolve_body(&RequestBodyDraft::Json {
                text: "{\"x\":\"{{missing}}\"}".to_string(),
            })
            .expect_err("should fail for missing variable");
        assert!(error.to_string().contains("missing"));
    }
}
