use std::{fs::OpenOptions, io::Write, path::PathBuf};

use anyhow::{Context, Result};
use chrono::{Datelike, Local, Timelike};
use serde_json::Value;

use crate::{
    config::paths::AppPaths, runtime::store::RuntimeStore, services::resolver::ResolveContext,
};

pub fn ensure_log_file(store: &RuntimeStore, project: &str) -> Result<PathBuf> {
    if let Some(active) = store.active_log(project) {
        let now = Local::now();
        let expected_name = format!(
            "{:04}{:02}{:02}000000.md",
            now.year(),
            now.month(),
            now.day()
        );
        if active
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == expected_name)
        {
            return Ok(active);
        }
    }

    let now = Local::now();
    let file_name = format!(
        "{:04}{:02}{:02}000000.md",
        now.year(),
        now.month(),
        now.day()
    );
    let path = project_log_dir(&store.paths, project)?.join(file_name);
    touch(&path)?;
    store.set_active_log(project, path.clone());
    Ok(path)
}

pub fn create_manual_log_file(store: &RuntimeStore, project: &str) -> Result<PathBuf> {
    let now = Local::now();
    let file_name = format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}.md",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );
    let path = project_log_dir(&store.paths, project)?.join(file_name);
    touch(&path)?;
    store.set_active_log(project, path.clone());
    Ok(path)
}

pub fn append_request_log(
    store: &RuntimeStore,
    project: &str,
    resolver: &ResolveContext,
    curl_command: &str,
    status_code: u16,
    response_body: &str,
) -> Result<PathBuf> {
    let file = ensure_log_file(store, project)?;
    let mut text = log_block(curl_command, status_code, response_body);

    for (value, mask) in resolver.masks() {
        if !value.is_empty() {
            text = text.replace(&value, &mask);
        }
    }

    let mut handle = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)
        .with_context(|| format!("failed to open {}", file.display()))?;
    handle
        .write_all(text.as_bytes())
        .with_context(|| format!("failed to append {}", file.display()))?;

    Ok(file)
}

pub fn append_raw_log(
    store: &RuntimeStore,
    project: &str,
    curl_command: &str,
    status_code: u16,
    response_body: &str,
) -> Result<PathBuf> {
    let file = ensure_log_file(store, project)?;
    let text = log_block(curl_command, status_code, response_body);

    let mut handle = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)
        .with_context(|| format!("failed to open {}", file.display()))?;
    handle
        .write_all(text.as_bytes())
        .with_context(|| format!("failed to append {}", file.display()))?;

    Ok(file)
}

fn project_log_dir(paths: &AppPaths, project: &str) -> Result<PathBuf> {
    let dir = paths.logs_root.join(project);
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    Ok(dir)
}

fn touch(path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    Ok(())
}

fn log_block(curl_command: &str, status_code: u16, response_body: &str) -> String {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S JST");
    let response_body = format_response_body(response_body);
    format!("# {timestamp}\n```\n{curl_command}\n\n{status_code}\n{response_body}\n```\n\n")
}

fn format_response_body(response_body: &str) -> String {
    serde_json::from_str::<Value>(response_body)
        .ok()
        .and_then(|json| serde_json::to_string_pretty(&json).ok())
        .unwrap_or_else(|| response_body.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::Result;
    use tempfile::tempdir;

    use super::{append_raw_log, append_request_log};
    use crate::{
        config::paths::AppPaths, domain::http::AuthCredentials,
        models::runtime::RuntimeEnvironmentState, runtime::store::RuntimeStore,
        services::resolver::ResolveContext,
    };

    #[test]
    fn append_request_log_masks_values_and_writes_markdown_block() -> Result<()> {
        let tmp = tempdir()?;
        let paths = AppPaths::new(tmp.path())?;
        let store = RuntimeStore::load(paths)?;
        let resolver = ResolveContext {
            environment: RuntimeEnvironmentState {
                constants: HashMap::new(),
                variables: HashMap::from([("token".to_string(), "secret-token".to_string())]),
                masks: HashMap::from([("token".to_string(), "xxx".to_string())]),
            },
            auth: AuthCredentials::default(),
        };

        let file = append_request_log(
            &store,
            "project-1",
            &resolver,
            "curl -X POST 'http://example.test' \\\n  -H 'X-Token: secret-token' \\\n  -d 'token=secret-token'",
            200,
            "{\"token\":\"secret-token\"}",
        )?;

        let text = std::fs::read_to_string(file)?;
        assert!(text.contains("```\ncurl -X POST"));
        assert!(text.contains("\n\n200\n"));
        assert!(text.contains("-d 'token=xxx'"));
        assert!(text.contains("X-Token: xxx"));
        assert!(text.contains("{\n  \"token\": \"xxx\"\n}"));
        assert!(!text.contains("```bash"));
        assert!(!text.contains("secret-token"));
        Ok(())
    }

    #[test]
    fn append_raw_log_does_not_mask_values() -> Result<()> {
        let tmp = tempdir()?;
        let paths = AppPaths::new(tmp.path())?;
        let store = RuntimeStore::load(paths)?;

        let file = append_raw_log(
            &store,
            "project-1",
            "curl -X POST 'http://example.test' \\\n  -H 'X-Token: secret-token'",
            201,
            "{\"token\":\"secret-token\"}",
        )?;

        let text = std::fs::read_to_string(file)?;
        assert!(text.contains("secret-token"));
        assert!(text.contains("\n\n201\n"));
        assert!(!text.contains("```bash"));
        Ok(())
    }

    #[test]
    fn append_raw_log_keeps_non_json_response_body_as_is() -> Result<()> {
        let tmp = tempdir()?;
        let paths = AppPaths::new(tmp.path())?;
        let store = RuntimeStore::load(paths)?;

        let file = append_raw_log(
            &store,
            "project-1",
            "curl -X GET 'http://example.test'",
            200,
            "plain text response",
        )?;

        let text = std::fs::read_to_string(file)?;
        assert!(text.contains("plain text response"));
        Ok(())
    }
}
