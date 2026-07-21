use axum::{
    Router, routing::get,
};
use tokio::net::TcpListener;
use tracing_subscriber::prelude::*;
use tracing::info;

pub async fn run() {
    // setup tracing (ตัวเดียวอยู่ ไม่ต้องใช้ env_logger แล้ว)
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        ))
        .init();

    // Router API Restful
    let app = Router::new()
        .route("/", get(|| async {
            "Welcome to the API SiamMock"
        }));
    
    // listen 
    let listener = TcpListener::bind("0.0.0.0:4300").await.unwrap_or_else(|_| panic!("Failed to bind to port 4300"));
    info!("Server is running on http://localhost:4300");

    // ✅ เติม .into_make_service() เพื่อให้เข้ากับฟังก์ชัน axum::serve
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap_or_else(|_| panic!("Failed to serve"));
}