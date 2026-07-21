mod app;
mod cli;
mod config;
mod data;
mod handlers;
mod response;
mod router;
mod validation;

use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() {
    let args = Cli::parse().start_args();
    app::run(args).await;
}
