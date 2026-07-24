use axum::{
    Router,
    routing::{any, get},
};

use crate::{
    config::schema::MockConfig,
    data::CsvStore,
    handlers::mock::{AppState, dispatch},
};

pub fn build_router(config: MockConfig, csv: CsvStore, data_dir: String) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { "Welcome to SiamMock — routes are loaded from your JSON config" }),
        )
        .fallback(any(dispatch))
        .with_state(AppState::new(config, csv, data_dir))
}
