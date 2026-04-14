use anyhow::Result;
use clap::Parser;

use vurl_backend::{app::build_app, cli::Cli, logging::init_tracing};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing()?;
    build_app(cli)?.run().await
}
