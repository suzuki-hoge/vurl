use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub defs_root: PathBuf,
    pub logs_root: PathBuf,
    pub frontend_dist_root: PathBuf,
}

impl AppPaths {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = expand_tilde(root.as_ref());
        let defs_root = root.join("defs");
        let logs_root = root.join("logs");
        let frontend_dist_root = project_root().join("src/frontend/dist");

        std::fs::create_dir_all(&defs_root)
            .with_context(|| format!("failed to create defs root: {}", defs_root.display()))?;
        std::fs::create_dir_all(&logs_root)
            .with_context(|| format!("failed to create logs root: {}", logs_root.display()))?;

        Ok(Self {
            root,
            defs_root,
            logs_root,
            frontend_dist_root,
        })
    }

    pub fn from_default_root() -> Result<Self> {
        Self::new(default_root())
    }
}

fn expand_tilde(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    if (raw == "~" || raw.starts_with("~/"))
        && let Some(home) = std::env::var_os("HOME")
    {
        if raw == "~" {
            return PathBuf::from(home);
        }
        return PathBuf::from(home).join(raw.trim_start_matches("~/"));
    }

    path.to_path_buf()
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)
        .expect("project root should exist")
}

fn default_root() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".vurl")
}
