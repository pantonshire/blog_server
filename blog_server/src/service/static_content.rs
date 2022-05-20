use std::{
    convert::Infallible,
    io,
    path::Path,
};

use axum::{
    body::Body,
    handler::Handler,
    http::Uri,
    routing::{get_service, MethodRouter},
};
use libshire::convert::Empty;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tracing::{info, error};

use super::response::Error;

pub fn service(static_dir: &Path) -> MethodRouter<Body, Infallible> {
    let fallback_service = handle_fallback
        .into_service()
        .map_err(Empty::elim::<io::Error>);
    
    let serve_dir = ServeDir::new(static_dir)
        .fallback(fallback_service);

    get_service(serve_dir)
        .handle_error(handle_error)
}

pub async fn handle_fallback(uri: Uri) -> Error {
    info!(path = %uri.path(), "Requested static file not found");
    Error::StaticResourceNotFound
}

pub async fn handle_error(uri: Uri, err: io::Error) -> Error {
    error!(path = %uri.path(), err = %err, "IO error");
    Error::Internal
}
