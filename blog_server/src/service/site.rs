use std::sync::Arc;

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

use crate::{
    Config,
    posts_store::ConcurrentPostsStore
};
use super::{
    atom,
    contact,
    index,
    post,
    posts_list,
    response::Error,
    rss,
    static_content,
};

pub fn service(
    config: Arc<Config>,
    posts_store: ConcurrentPostsStore,
) -> Router
{
    Router::new()
        .route("/", get(index::handle))
        .route("/rss.xml", get(rss::handle))
        .route("/atom.xml", get(atom::handle))
        .route("/contact", get(contact::handle))
        .route("/articles", get(posts_list::handle))
        .route("/articles/:post_id", get(post::handle))
        .nest("/static", static_content::service(&config.static_dir))
        .fallback(handle_fallback.into_service())
        .layer(ConcurrencyLimitLayer::new(config.concurrency_limit))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(config))
        .layer(Extension(posts_store))
}

pub async fn handle_fallback(uri: Uri) -> Error {
    info!(path = %uri.path(), "Requested resource not found");
    Error::RouteNotFound
}
