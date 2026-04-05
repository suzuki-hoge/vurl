use std::{fs::OpenOptions, io::Write, path::PathBuf};

use anyhow::{Context, Result};
use chrono::{Datelike, Local, Timelike};

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
    response_body: &str,
) -> Result<PathBuf> {
    let file = ensure_log_file(store, project)?;
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S JST");
    let mut text = format!("# {timestamp}\n```bash\n{curl_command}\n{response_body}\n```\n\n");

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
