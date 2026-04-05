use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(name = "vurl-backend")]
pub struct Cli {
    #[arg(long, hide = true, default_value_t = false)]
    pub child: bool,
}
