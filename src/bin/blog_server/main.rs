mod config;
mod fs_watcher;
mod render;
mod service;
mod template;

use std::{env, fs, sync::Arc, thread};

use axum::Server;
use miette::{IntoDiagnostic, Context};
use tracing::info;

use blog::{
    codeblock::CodeBlockRenderer,
    db::ConcurrentPostsStore,
};

use config::Config;
use render::Renderer;

fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();

    // Load the configuration from the KDL config file specified by the first command-line
    // argument.
    let config = Arc::new({
        let config_path = env::args().nth(1)
            .ok_or_else(|| miette::Error::msg("No config file specified"))?;

        info!(path = %config_path, "Loading config");

        let contents = fs::read_to_string(&config_path)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to read config file {}", config_path))?;
            
        knuffel::parse::<Config>(&config_path, &contents)
            .wrap_err_with(|| format!("Failed to parse config file {}", config_path))?
    });

    // Create the data structure used to store the rendered posts. This uses an `Arc` internally,
    // so clones will point to the same underlying data.
    let posts_store = ConcurrentPostsStore::new();

    let code_renderer = CodeBlockRenderer::new();

    // Create the post renderer and the mpsc channel that will be used to communicate with it.
    let (renderer, tx) = Renderer::new(
        config.clone(),
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

async fn run(
    config: Arc<Config>,
    posts_store: ConcurrentPostsStore,
) -> miette::Result<()>
{
    let bind_address = &config.bind
        .parse()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to parse socket address \"{}\"", config.bind))?;

    let service = service::site_service(config, posts_store);

    info!(address = %bind_address, "Starting server");

    Server::try_bind(bind_address)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to bind {}", bind_address))?
        .serve(service.into_make_service())
        .await
        .into_diagnostic()
        .wrap_err("Fatal error while running the server")
}
