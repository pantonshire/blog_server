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
use mime::Mime;
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{info, error};

use super::response::Error;

pub(super) fn file_service(file_path: &Path, mime: Option<&Mime>) -> MethodRouter<Body, Infallible> {
    let serve_file = match mime {
        Some(mime) => ServeFile::new_with_mime(file_path, mime),
        None => ServeFile::new(file_path),
    };

    get_service(serve_file)
        .handle_error(handle_error)
}

pub(super) fn dir_service(dir_path: &Path) -> MethodRouter<Body, Infallible> {
    let fallback_service = handle_fallback
        .into_service()
        .map_err(Empty::elim::<io::Error>);
    
    let serve_dir = ServeDir::new(dir_path)
        .fallback(fallback_service);

    get_service(serve_dir)
        .handle_error(handle_error)
}

pub(super) async fn handle_fallback(uri: Uri) -> Error {
    info!(path = %uri.path(), "Requested static file not found");
    Error::StaticResourceNotFound
}

pub(super) async fn handle_error(uri: Uri, err: io::Error) -> Error {
    error!(path = %uri.path(), err = %err, "IO error");
    Error::Internal
}
