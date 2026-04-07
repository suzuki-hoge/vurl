use std::{
    io::{self, Read},
    process::{Child, Command, Stdio},
};

use anyhow::{Context, Result};

use crate::{
    app::build_app, cli::Cli, config::paths::AppPaths, runtime::store::RuntimeStore,
    services::logging::create_manual_log_file,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SupervisorCommand {
    CheckYaml,
    RotateLogs,
    RestartBackend,
    Quit,
    Ignore(char),
}

pub async fn run_child(cli: Cli) -> Result<()> {
    let app = build_app(cli)?;
    app.run().await
}

pub fn run_parent(cli: Cli) -> Result<()> {
    let mut child = spawn_child(&cli)?;

    eprintln!("vurl-backend supervisor started");
    eprintln!("commands: c=check yaml, l=new log file, r=restart backend, q=quit");

    let stdin = io::stdin();
    for byte in stdin.lock().bytes() {
        let byte = match byte {
            Ok(value) => value,
            Err(error) => {
                stop_child(&mut child)?;
                return Err(error).context("failed to read stdin");
            }
        };

        match parse_command(byte as char) {
            SupervisorCommand::CheckYaml => run_check(&cli)?,
            SupervisorCommand::RotateLogs => rotate_logs(&cli)?,
            SupervisorCommand::RestartBackend => {
                stop_child(&mut child)?;
                child = spawn_child(&cli)?;
                eprintln!("backend restarted");
            }
            SupervisorCommand::Quit => {
                stop_child(&mut child)?;
                eprintln!("backend stopped");
                return Ok(());
            }
            SupervisorCommand::Ignore(other) => {
                eprintln!("ignored input: {other}");
            }
        }
    }

    stop_child(&mut child)?;
    Ok(())
}

fn run_check(cli: &Cli) -> Result<()> {
    let _ = cli;
    let project_count = load_store()?.project_names().len();
    eprintln!("yaml check ok: {project_count} project(s)");
    Ok(())
}

fn rotate_logs(cli: &Cli) -> Result<()> {
    let _ = cli;
    let rotated = rotate_logs_for_store(load_store()?.as_ref())?;
    eprintln!("log rotated: {rotated} project(s)");
    Ok(())
}

fn spawn_child(_cli: &Cli) -> Result<Child> {
    let current_exe = std::env::current_exe().context("failed to resolve current exe")?;
    let mut command = Command::new(current_exe);
    command
        .arg("--child")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let child = command.spawn().context("failed to spawn child backend")?;
    eprintln!("backend child started");
    Ok(child)
}

fn stop_child(child: &mut Child) -> Result<()> {
    match child.try_wait()? {
        Some(_) => Ok(()),
        None => {
            child.kill().context("failed to kill child")?;
            child.wait().context("failed to wait for child stop")?;
            Ok(())
        }
    }
}

fn parse_command(input: char) -> SupervisorCommand {
    match input {
        'c' => SupervisorCommand::CheckYaml,
        'l' => SupervisorCommand::RotateLogs,
        'r' => SupervisorCommand::RestartBackend,
        'q' => SupervisorCommand::Quit,
        '\n' | '\r' | ' ' | '\t' => SupervisorCommand::Ignore(input),
        other => SupervisorCommand::Ignore(other),
    }
}

fn load_store() -> Result<std::sync::Arc<RuntimeStore>> {
    let paths = AppPaths::from_default_root()?;
    RuntimeStore::load(paths)
}

fn rotate_logs_for_store(store: &RuntimeStore) -> Result<usize> {
    let projects = store.project_names();
    for project in &projects {
        create_manual_log_file(store, project)?;
    }
    Ok(projects.len())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use tempfile::tempdir;

    use super::{SupervisorCommand, parse_command, rotate_logs_for_store};
    use crate::{config::paths::AppPaths, runtime::store::RuntimeStore};

    #[test]
    fn parses_supervisor_commands() {
        assert_eq!(parse_command('c'), SupervisorCommand::CheckYaml);
        assert_eq!(parse_command('l'), SupervisorCommand::RotateLogs);
        assert_eq!(parse_command('r'), SupervisorCommand::RestartBackend);
        assert_eq!(parse_command('q'), SupervisorCommand::Quit);
        assert_eq!(parse_command(' '), SupervisorCommand::Ignore(' '));
        assert_eq!(parse_command('x'), SupervisorCommand::Ignore('x'));
    }

    #[test]
    fn rotates_logs_for_each_project() -> Result<()> {
        let tmp = tempdir()?;
        let root = tmp.path();
        fs::create_dir_all(root.join("defs/project-a/environments"))?;
        fs::create_dir_all(root.join("defs/project-b/environments"))?;
        fs::write(
            root.join("defs/project-a/environments/auth.yaml"),
            "environments: {}\n",
        )?;
        fs::write(
            root.join("defs/project-b/environments/auth.yaml"),
            "environments: {}\n",
        )?;

        let store = RuntimeStore::load(AppPaths::new(root)?)?;
        let rotated = rotate_logs_for_store(store.as_ref())?;

        assert_eq!(rotated, 2);
        assert!(store.active_log("project-a").is_some());
        assert!(store.active_log("project-b").is_some());
        Ok(())
    }
}
