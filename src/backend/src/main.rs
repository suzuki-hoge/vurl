use anyhow::Result;
use clap::Parser;

use vurl_backend::{cli::Cli, logging::init_tracing, process::supervisor};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing()?;

    if cli.child {
        supervisor::run_child(cli).await
    } else {
        supervisor::run_parent(cli)
    }
}
