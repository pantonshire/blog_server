mod codeblock;
mod fs_watcher;
mod post;
mod posts_store;
mod render;
mod service;
mod template;

use std::{env, fs, path::PathBuf, thread};

use axum::Server;
use miette::{IntoDiagnostic, Context};
use tracing::info;

use codeblock::CodeBlockRenderer;
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
    let service = service::site_service(
        posts_store,
        &config.static_dir,
        config.concurrency_limit
    );

    let bind_address = &config.bind
        .parse()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to parse socket address \"{}\"", config.bind))?;

    info!(address = %bind_address, "Starting server");

    Server::try_bind(bind_address)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to bind {}", bind_address))?
        .serve(service.into_make_service())
        .await
        .into_diagnostic()
        .wrap_err("Fatal error while running the server")
}
