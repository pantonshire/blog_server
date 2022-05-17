mod codeblock;
mod fs_watcher;
mod handlers;
mod html_response;
mod post;
mod posts_store;
mod render;

use std::{env, fs, io, path::PathBuf, thread};

use axum::{
    {routing::{get, get_service}, Router},
    extract::{Extension, Path},
    response::{IntoResponse, Response},
    handler::Handler,
    http::StatusCode
};
use libshire::convert::infallible_elim;
use maud::html;
use miette::{IntoDiagnostic, Context};
use tower::{
    limit::ConcurrencyLimitLayer,
    ServiceExt,
};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;

use codeblock::CodeBlockRenderer;
use html_response::HtmlResponse;
use posts_store::ConcurrentPostsStore;
use render::Renderer;

#[derive(knuffel::Decode)]
struct Config {
    #[knuffel(child, unwrap(argument))]
    bind: String,
    #[knuffel(child, unwrap(argument))]
    posts_dir: PathBuf,
    #[knuffel(child, unwrap(argument))]
    static_dir: PathBuf,
    #[knuffel(child, unwrap(argument))]
    concurrency_limit: usize,
}

fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();

    // Load the configuration from the KDL config file specified by the first command-line
    // argument.
    let config = {
        let config_path = env::args().nth(1)
            .ok_or_else(|| miette::Error::msg("No config file specified"))?;

        info!(path = %config_path, "Loading config");

        let contents = fs::read_to_string(&config_path)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to read config file {}", config_path))?;
            
        knuffel::parse::<Config>(&config_path, &contents)
            .wrap_err_with(|| format!("Failed to parse config file {}", config_path))?
    };

    // Create the data structure used to store the rendered posts. This uses an `Arc` internally,
    // so clones will point to the same underlying data.
    let posts_store = ConcurrentPostsStore::new();

    let code_renderer = CodeBlockRenderer::new();

    // Create the post renderer and the mpsc channel that will be used to communicate with it.
    let (renderer, tx) = Renderer::new(
        posts_store.clone(),
        code_renderer,
        config.posts_dir.clone()
    );

    // Dropping the watcher stops its thread, so keep it alive until `main` returns.
    let _watcher = fs_watcher::start_watching(tx, &config.posts_dir)?;

    thread::spawn(move || {
        renderer.handle_events();
    });

    info!("Started renderer thread");

    // To run the web server, we need to be in an async context, so create a new Tokio runtime and
    // pass control to it.
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .into_diagnostic()
        .wrap_err("Failed to create async runtime")?
        .block_on(run(config, posts_store))
}

async fn run(config: Config, posts_store: ConcurrentPostsStore) -> miette::Result<()> {
    let static_service = get_service(ServeDir::new(&config.static_dir)
            .fallback(handle_fallback
                .into_service()
                .map_err(infallible_elim::<io::Error>)))
        .handle_error(handle_static_io_error);

    let router = Router::new()
        .route("/", get(handle_index))
        .route("/posts/:post_id", get(handle_post_page))
        .nest("/static", static_service)
        .fallback(handle_fallback.into_service())
        .layer(ConcurrencyLimitLayer::new(config.concurrency_limit))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(posts_store));

    let bind_address = &config.bind
        .parse()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to parse socket address \"{}\"", config.bind))?;

    info!(address = %bind_address, "Starting server");

    axum::Server::try_bind(bind_address)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to bind {}", bind_address))?
        .serve(router.into_make_service())
        .await
        .into_diagnostic()
        .wrap_err("Fatal error while running the server")
}

async fn handle_fallback() -> Error {
    Error::NotFound
}

async fn handle_static_io_error(_err: io::Error) -> Error {
    Error::Internal
}

async fn handle_index(Extension(posts): Extension<ConcurrentPostsStore>) -> HtmlResponse {
    HtmlResponse::new()
        .with_title_static("Placeholder title")
        .with_crawler_permissive()
        .with_body(html! {
            h1 { "Here is my great heading" }
            p { "Hello world" }
            ul {
                @for post in posts.read().await.iter_by_created().rev() {
                    li {
                        a href={ "/posts/" (post.id_str()) } {
                            (post.title())
                        };
                    }
                }
            }
        })
}

async fn handle_post_page(
    Path(post_id): Path<String>,
    Extension(posts): Extension<ConcurrentPostsStore>
) -> Result<HtmlResponse, Error>
{
    let post = posts.get(&post_id)
        .await
        .ok_or(Error::NotFound)?;

    Ok(HtmlResponse::new()
        .with_crawler_permissive()
        .with_title_owned(post.title().to_owned())
        .with_head(html! {
            link href="/static/style/code.css" rel="stylesheet";
        })
        .with_body(html! {
            h1 { (post.title()) }
            p { "by " (post.author()) }
            article {
                (post.html())
            }
        }))
}

// TODO: store diagnostic information in Error struct which is output to trace
#[derive(Debug)]
enum Error {
    Internal,
    NotFound,
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Error::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        
        // Create a string buffer containing the full error text, e.g. "404 Not Found".
        let status_text = {
            let status_code_str = status_code.as_str();
            let reason = status_code.canonical_reason();
            let mut buf = String::with_capacity(
                status_code_str.len() + reason.map(|reason| reason.len() + 1).unwrap_or(0));
            buf.push_str(status_code_str);
            if let Some(reason) = reason {
                buf.push(' ');
                buf.push_str(reason);
            }
            buf
        };

        HtmlResponse::new()
            .with_status(status_code)
            .with_body(html! {
                p { (status_text) }
            })
            .with_title_owned(status_text)
            .into_response()
    }
}
