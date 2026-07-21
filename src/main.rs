mod app;
mod config;

#[tokio::main]

async fn main() {
    app::run().await;
}