use crate::domain::models::system::SystemStats;
use crate::usecase::system::SystemUsecase;
use axum::{extract::State, routing::get, Json, Router};
use std::sync::Arc;

pub async fn get_system_stats(State(usecase): State<Arc<SystemUsecase>>) -> Json<SystemStats> {
    let stats = usecase.get_stats().await.unwrap_or(SystemStats {
        cpu_count: 0,
        memory_total_kb: 0,
        memory_available_kb: 0,
        memory_used_percent: 0.0,
        disk_total_bytes: 0,
        disk_available_bytes: 0,
        disk_used_percent: 0.0,
        uptime_seconds: 0,
        load_average: crate::domain::models::LoadAverage {
            one: 0.0,
            five: 0.0,
            fifteen: 0.0,
        },
    });
    Json(stats)
}

pub fn system_routes(usecase: Arc<SystemUsecase>) -> Router {
    Router::new()
        .route("/stats", get(get_system_stats))
        .with_state(usecase)
}
