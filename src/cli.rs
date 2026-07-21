use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "siammock", version, about = "Mock API and Webhook Simulator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the mock API server
    Start(StartArgs),
}

#[derive(Args, Clone)]
pub struct StartArgs {
    /// Mock config: file, directory, or comma-separated paths
    #[arg(short, long, default_value = "mock")]
    pub config: String,

    /// Server port
    #[arg(short, long, default_value_t = 4300)]
    pub port: u16,

    /// Server host
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    /// CSV data directory (database exports)
    #[arg(long, default_value = "data")]
    pub data: String,
}

impl Cli {
    pub fn start_args(self) -> StartArgs {
        match self.command {
            Some(Commands::Start(args)) => args,
            None => StartArgs {
                config: "mock".into(),
                port: 4300,
                host: "0.0.0.0".into(),
                data: "data".into(),
            },
        }
    }
}
