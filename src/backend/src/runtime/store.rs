use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result, bail};

use crate::{
    config::paths::AppPaths,
    domain::{
        auth::AuthDefinitions,
        environment::EnvironmentDefinition,
        project::{ProjectData, RequestTreeNode},
        request::RequestDefinition,
    },
    models::runtime::RuntimeEnvironmentState,
};

#[derive(Debug)]
pub struct RuntimeStore {
    pub paths: AppPaths,
    projects: HashMap<String, ProjectData>,
    env_state: Mutex<HashMap<String, HashMap<String, RuntimeEnvironmentState>>>,
    active_logs: Mutex<HashMap<String, PathBuf>>,
}

impl RuntimeStore {
    pub fn load(paths: AppPaths) -> Result<Arc<Self>> {
        let mut projects = HashMap::new();
        let mut env_state = HashMap::new();

        if paths.defs_root.exists() {
            for entry in std::fs::read_dir(&paths.defs_root)
                .with_context(|| format!("failed to read {}", paths.defs_root.display()))?
            {
                let entry = entry?;
                if !entry.file_type()?.is_dir() {
                    continue;
                }

                let project_name = entry.file_name().to_string_lossy().to_string();
                let project_dir = entry.path();
                let requests = load_requests(&project_dir.join("requests"))?;
                let environments = load_environments(&project_dir.join("environments"))?;
                let auth = load_auth(&project_dir.join("environments").join("auth.yaml"))?;

                let state_map = environments
                    .iter()
                    .map(|(name, env)| (name.clone(), RuntimeEnvironmentState::from(env)))
                    .collect();

                env_state.insert(project_name.clone(), state_map);
                projects.insert(
                    project_name.clone(),
                    ProjectData {
                        name: project_name,
                        requests,
                        environments,
                        auth,
                    },
                );
            }
        }

        Ok(Arc::new(Self {
            paths,
            projects,
            env_state: Mutex::new(env_state),
            active_logs: Mutex::new(HashMap::new()),
        }))
    }

    pub fn project_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.projects.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn project(&self, name: &str) -> Result<&ProjectData> {
        self.projects
            .get(name)
            .with_context(|| format!("project not found: {name}"))
    }

    pub fn tree(&self, project: &str) -> Result<Vec<RequestTreeNode>> {
        let project = self.project(project)?;
        let mut root = DirectoryNode::default();

        for (path, definition) in &project.requests {
            insert_tree(&mut root, path, definition);
        }

        Ok(root.into_children(String::new()))
    }

    pub fn request_definition(&self, project: &str, path: &str) -> Result<RequestDefinition> {
        self.project(project)?
            .requests
            .get(path)
            .cloned()
            .with_context(|| format!("request not found: {project}/{path}"))
    }

    pub fn environment_names(&self, project: &str) -> Result<Vec<String>> {
        let project = self.project(project)?;
        let mut names: Vec<_> = project.environments.keys().cloned().collect();
        names.sort();
        Ok(names)
    }

    pub fn env_state(&self, project: &str, environment: &str) -> Result<RuntimeEnvironmentState> {
        let guard = self.env_state.lock().expect("env_state poisoned");
        guard
            .get(project)
            .and_then(|by_env| by_env.get(environment))
            .cloned()
            .with_context(|| format!("environment not found: {project}/{environment}"))
    }

    pub fn update_env_variables(
        &self,
        project: &str,
        environment: &str,
        updates: &HashMap<String, String>,
    ) -> Result<RuntimeEnvironmentState> {
        let mut guard = self.env_state.lock().expect("env_state poisoned");
        let state = guard
            .get_mut(project)
            .and_then(|by_env| by_env.get_mut(environment))
            .with_context(|| format!("environment not found: {project}/{environment}"))?;

        for (key, value) in updates {
            state.variables.insert(key.clone(), value.clone());
        }

        Ok(state.clone())
    }

    pub fn set_active_log(&self, project: &str, file: PathBuf) {
        let mut guard = self.active_logs.lock().expect("active_logs poisoned");
        guard.insert(project.to_string(), file);
    }

    pub fn active_log(&self, project: &str) -> Option<PathBuf> {
        self.active_logs
            .lock()
            .expect("active_logs poisoned")
            .get(project)
            .cloned()
    }
}

fn load_requests(dir: &Path) -> Result<HashMap<String, RequestDefinition>> {
    let mut items = HashMap::new();
    if !dir.exists() {
        return Ok(items);
    }

    visit_yaml_files(dir, |relative_path, full_path| {
        let definition: RequestDefinition = serde_yaml::from_str(
            &std::fs::read_to_string(full_path)
                .with_context(|| format!("failed to read {}", full_path.display()))?,
        )
        .with_context(|| format!("failed to parse {}", full_path.display()))?;
        items.insert(relative_path, definition);
        Ok(())
    })?;

    Ok(items)
}

fn load_environments(dir: &Path) -> Result<HashMap<String, EnvironmentDefinition>> {
    let mut items = HashMap::new();
    if !dir.exists() {
        return Ok(items);
    }

    for entry in
        std::fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        if path.file_name().and_then(|name| name.to_str()) == Some("auth.yaml") {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }

        let env: EnvironmentDefinition = serde_yaml::from_str(
            &std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?,
        )
        .with_context(|| format!("failed to parse {}", path.display()))?;

        items.insert(env.name.clone(), env);
    }

    Ok(items)
}

fn load_auth(path: &Path) -> Result<AuthDefinitions> {
    if !path.exists() {
        bail!("auth definition not found: {}", path.display());
    }

    serde_yaml::from_str(
        &std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", path.display()))
}

fn visit_yaml_files(root: &Path, mut visit: impl FnMut(String, &Path) -> Result<()>) -> Result<()> {
    fn walk(
        base: &Path,
        dir: &Path,
        visit: &mut impl FnMut(String, &Path) -> Result<()>,
    ) -> Result<()> {
        for entry in
            std::fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                walk(base, &path, visit)?;
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
                continue;
            }

            let relative = path
                .strip_prefix(base)
                .expect("path should be under base")
                .to_string_lossy()
                .replace('\\', "/");
            visit(relative, &path)?;
        }
        Ok(())
    }

    walk(root, root, &mut visit)
}

#[derive(Default)]
struct DirectoryNode {
    directories: HashMap<String, DirectoryNode>,
    requests: Vec<RequestLeaf>,
}

#[derive(Debug)]
struct RequestLeaf {
    name: String,
    path: String,
    title: String,
    method: String,
}

fn insert_tree(root: &mut DirectoryNode, path: &str, definition: &RequestDefinition) {
    let parts: Vec<_> = path.split('/').collect();
    let mut current = root;

    for part in &parts[..parts.len().saturating_sub(1)] {
        current = current.directories.entry((*part).to_string()).or_default();
    }

    current.requests.push(RequestLeaf {
        name: parts.last().copied().unwrap_or(path).to_string(),
        path: path.to_string(),
        title: definition.name.clone(),
        method: definition.method.clone(),
    });
}

impl DirectoryNode {
    fn into_children(mut self, base_path: String) -> Vec<RequestTreeNode> {
        let mut directory_names: Vec<_> = self.directories.keys().cloned().collect();
        directory_names.sort();

        let mut nodes = Vec::new();
        for name in directory_names {
            let child = self.directories.remove(&name).expect("directory exists");
            let child_path = if base_path.is_empty() {
                name.clone()
            } else {
                format!("{base_path}/{name}")
            };
            nodes.push(RequestTreeNode::Directory {
                name,
                path: child_path.clone(),
                children: child.into_children(child_path),
            });
        }

        self.requests.sort_by(|a, b| a.path.cmp(&b.path));
        for request in self.requests {
            nodes.push(RequestTreeNode::Request {
                name: request.name,
                path: request.path,
                title: request.title,
                method: request.method,
            });
        }

        nodes
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use tempfile::tempdir;

    use super::RuntimeStore;
    use crate::config::paths::AppPaths;

    #[test]
    fn load_project_tree_and_definition() -> Result<()> {
        let tmp = tempdir()?;
        let root = tmp.path();
        fs::create_dir_all(root.join("defs/project-1/requests/users"))?;
        fs::create_dir_all(root.join("defs/project-1/environments"))?;

        fs::write(
            root.join("defs/project-1/requests/users/get-user.yaml"),
            r#"
name: Get User
method: GET
path: /users/{{user_id}}
auth: true
request:
  query: []
  headers: []
  body:
    type: json
    text: ""
"#,
        )?;

        fs::write(
            root.join("defs/project-1/environments/local.yaml"),
            r#"
name: local
constants:
  base_url:
    value: http://localhost:18080
variables:
  user_id:
    value: "42"
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
      default:
        variables:
          auth_token: token
"#,
        )?;

        let paths = AppPaths::new(root)?;
        let store = RuntimeStore::load(paths)?;

        assert_eq!(store.project_names(), vec!["project-1".to_string()]);
        assert_eq!(
            store.environment_names("project-1")?,
            vec!["local".to_string()]
        );

        let tree = store.tree("project-1")?;
        assert_eq!(tree.len(), 1);

        let definition = store.request_definition("project-1", "users/get-user.yaml")?;
        assert_eq!(definition.name, "Get User");
        assert_eq!(definition.method, "GET");

        Ok(())
    }
}
