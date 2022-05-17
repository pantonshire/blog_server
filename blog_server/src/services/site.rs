use std::path::Path;

use axum::{
    handler::Handler,
    http::Uri,
    extract::Extension,
    Router,
    routing::get,
};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::posts_store::ConcurrentPostsStore;
use super::{
    index,
    posts,
    response::ErrorResponse,
    static_content,
};

pub fn service(
    posts_store: ConcurrentPostsStore,
    static_dir: &Path,
    concurrency_limit: usize
) -> Router
{
    Router::new()
        .route("/", get(index::handle))
        .route("/posts/:post_id", get(posts::handle))
        .nest("/static", static_content::service(static_dir))
        .fallback(handle_fallback.into_service())
        .layer(ConcurrencyLimitLayer::new(concurrency_limit))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(posts_store))
}

pub async fn handle_fallback(uri: Uri) -> ErrorResponse {
    info!(path = %uri.path(), "Requested resource not found");
    ErrorResponse::RouteNotFound
}
