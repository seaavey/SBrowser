use std::{sync::Arc, time::Instant};
use axum::{middleware, routing::get, Router};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    api::handlers::{
        handle_engines, handle_fetch_get, handle_fetch_post, handle_health,
        handle_search_get, handle_search_post, rate_limit_middleware, AppState,
    },
    config::Config,
    redis::RedisService,
    search::SearchService,
};

pub fn create_router(
    search_service: SearchService,
    config: Config,
    redis_service: Option<RedisService>,
) -> Router {
    let state = Arc::new(AppState {
        search_service,
        config,
        redis_service,
        start_time: Instant::now(),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(handle_health))
        .route("/health", get(handle_health))
        .route("/api/v1/engines", get(handle_engines))
        .route("/api/v1/search", get(handle_search_get).post(handle_search_post))
        .route("/api/v1/fetch", get(handle_fetch_get).post(handle_fetch_post))
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
