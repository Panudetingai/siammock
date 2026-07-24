use axum::Router;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::prelude::*;

use crate::cli::StartArgs;
use crate::config::loader::load_configs;
use crate::data::CsvStore;
use crate::router::builder::build_router;

pub async fn run(args: StartArgs) {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .init();

    let (config, files) = load_configs(&args.config).unwrap_or_else(|err| {
        panic!("Failed to load config from {}: {err}", args.config);
    });

    let file_list = files
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    info!(
        "Loaded {} route(s) from {} file(s): {file_list}",
        config.routes.len(),
        files.len()
    );

    let csv_store = CsvStore::load_from_dir(&args.data).unwrap_or_else(|err| {
        panic!("Failed to load csv data from {}: {err}", args.data);
    });

    if csv_store.loaded_files().is_empty() {
        info!("No CSV files found in {}", args.data);
    } else {
        info!(
            "Loaded CSV file(s) from {}: {}",
            args.data,
            csv_store.loaded_files().join(", ")
        );
    }

    let app: Router = build_router(config, csv_store, args.data);

    let addr = format!("{}:{}", "0.0.0.0", args.port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {addr}"));

    info!("Server is running on http://localhost:{}", args.port);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap_or_else(|_| panic!("Failed to serve"));
}
