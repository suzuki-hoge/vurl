use std::{
    io::{self, Read},
    process::{Child, Command, Stdio},
};

use anyhow::{Context, Result, bail};

use crate::{app::build_app, cli::Cli, config::paths::AppPaths, runtime::store::RuntimeStore};

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

        match byte as char {
            'c' => run_check(&cli)?,
            'l' => rotate_logs(&cli)?,
            'r' => {
                stop_child(&mut child)?;
                child = spawn_child(&cli)?;
                eprintln!("backend restarted");
            }
            'q' => {
                stop_child(&mut child)?;
                eprintln!("backend stopped");
                return Ok(());
            }
            '\n' | '\r' | ' ' | '\t' => {}
            other => {
                eprintln!("ignored input: {other}");
            }
        }
    }

    stop_child(&mut child)?;
    Ok(())
}

fn run_check(cli: &Cli) -> Result<()> {
    let _ = cli;
    let paths = AppPaths::from_default_root()?;
    let store = RuntimeStore::load(paths)?;
    eprintln!("yaml check ok: {} project(s)", store.project_names().len());
    Ok(())
}

fn rotate_logs(cli: &Cli) -> Result<()> {
    let _ = cli;
    let paths = AppPaths::from_default_root()?;
    let store = RuntimeStore::load(paths)?;
    let projects = store.project_names();

    for project in &projects {
        let status = Command::new("curl")
            .arg("-sS")
            .arg("-X")
            .arg("POST")
            .arg("http://127.0.0.1:1357/api/logs/new")
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(format!(r#"{{"project":"{project}"}}"#))
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("failed to request new log for {project}"))?;

        if !status.success() {
            bail!("backend rejected new log request for {project}");
        }
    }

    eprintln!("log rotated: {} project(s)", projects.len());
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
