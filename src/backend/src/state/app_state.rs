use std::sync::{Arc, RwLock};

use anyhow::Result;

use crate::runtime::store::RuntimeStore;

#[derive(Clone)]
pub struct AppState {
    store: Arc<RwLock<Arc<RuntimeStore>>>,
    pub backend_url: String,
}

impl AppState {
    pub fn new(store: Arc<RuntimeStore>, backend_url: String) -> Self {
        Self {
            store: Arc::new(RwLock::new(store)),
            backend_url,
        }
    }

    pub fn store(&self) -> Arc<RuntimeStore> {
        Arc::clone(
            &self
                .store
                .read()
                .expect("runtime store lock should not be poisoned"),
        )
    }

    pub fn reload(&self) -> Result<Arc<RuntimeStore>> {
        let paths = self.store().paths.clone();
        let next_store = RuntimeStore::load(paths)?;
        *self
            .store
            .write()
            .expect("runtime store lock should not be poisoned") = Arc::clone(&next_store);
        Ok(next_store)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use tempfile::tempdir;

    use super::AppState;
    use crate::{config::paths::AppPaths, runtime::store::RuntimeStore};

    fn write_project(root: &std::path::Path, request_name: &str) -> Result<()> {
        fs::create_dir_all(root.join("defs/project-1/requests"))?;
        fs::create_dir_all(root.join("defs/project-1/environments"))?;
        fs::write(
            root.join("defs/project-1/requests/get-user.yaml"),
            format!(
                r#"
name: {request_name}
method: GET
path: /users
auth: false
request:
  query: []
  headers: []
  body:
    type: json
    text: ""
"#
            ),
        )?;
        fs::write(
            root.join("defs/project-1/environments/local.yaml"),
            r#"
name: local
constants:
  base_url:
    value: http://localhost:18080
variables: {}
"#,
        )?;
        fs::write(
            root.join("defs/project-1/environments/auth.yaml"),
            r#"
environments:
  local:
    mode: fixed
    credentials:
      presets: []
    mappings:
      items: []
"#,
        )?;
        Ok(())
    }

    #[test]
    fn reload_replaces_store_after_successful_load() -> Result<()> {
        let tmp = tempdir()?;
        write_project(tmp.path(), "Before Reload")?;

        let store = RuntimeStore::load(AppPaths::new(tmp.path())?)?;
        let state = AppState::new(store, "http://127.0.0.1:1357".to_string());
        assert_eq!(
            state
                .store()
                .request_definition("project-1", "get-user.yaml")?
                .name,
            "Before Reload"
        );

        write_project(tmp.path(), "After Reload")?;
        state.reload()?;

        assert_eq!(
            state
                .store()
                .request_definition("project-1", "get-user.yaml")?
                .name,
            "After Reload"
        );
        Ok(())
    }

    #[test]
    fn reload_keeps_previous_store_when_new_yaml_is_invalid() -> Result<()> {
        let tmp = tempdir()?;
        write_project(tmp.path(), "Stable Request")?;

        let store = RuntimeStore::load(AppPaths::new(tmp.path())?)?;
        let state = AppState::new(store, "http://127.0.0.1:1357".to_string());

        fs::write(
            tmp.path().join("defs/project-1/requests/get-user.yaml"),
            "name: broken: yaml",
        )?;

        assert!(state.reload().is_err());
        assert_eq!(
            state
                .store()
                .request_definition("project-1", "get-user.yaml")?
                .name,
            "Stable Request"
        );
        Ok(())
    }
}
