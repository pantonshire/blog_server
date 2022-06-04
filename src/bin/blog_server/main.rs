mod config;
mod fs_watcher;
mod render;
mod service;
mod template;

use std::{
    env,
    error,
    fmt,
    fs,
    io,
    net::SocketAddr,
    path::PathBuf,
    process,
    sync::Arc,
    thread,
};

use hyper::Server;
use tracing::info;

use blog::{
    codeblock::CodeBlockRenderer,
    db::ConcurrentPostsStore,
};

use config::Config;
use render::Renderer;

fn main() {
    if let Err(err) = run() {
        eprintln!("***** Fatal error *****");
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    // Load the configuration from the TOML config file specified by the first command-line
    // argument.
    let config = Arc::new({
        let config_path = env::args().nth(1)
            .ok_or(Error::NoConfig)?;

        info!(path = %config_path, "Loading config");

        let contents = fs::read_to_string(&config_path)
            .map_err(Error::ReadConfig)?;
            
        contents.parse::<Config>()
            .map_err(Error::BadConfig)?
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
        .map_err(Error::TokioRuntime)?
        .block_on(run_server(config, posts_store))
}

async fn run_server(
    config: Arc<Config>,
    posts_store: ConcurrentPostsStore,
) -> Result<(), Error>
{
    let service = service::site_service(config.clone(), posts_store);

    info!(address = %config.bind, "Starting server");

    Server::try_bind(&config.bind)
        .map_err(|err| Error::Bind(config.bind, err))?
        .serve(service.into_make_service())
        .await
        .map_err(Error::Server)
}

#[derive(Debug)]
enum Error {
    NoConfig,
    ReadConfig(io::Error),
    BadConfig(toml::de::Error),
    CreateWatcher(notify::Error),
    WatchDir(PathBuf, notify::Error),
    TokioRuntime(io::Error),
    Bind(SocketAddr, hyper::Error),
    Server(hyper::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoConfig => {
                write!(f, "no config file specified")
            },
            Self::ReadConfig(err) => {
                write!(f, "failed to read config file: {}", err)
            },
            Self::BadConfig(err) => {
                write!(f, "error in config: {}", err)
            },
            Self::CreateWatcher(err) => {
                write!(f, "failed to create filesystem watcher: {}", err)
            },
            Self::WatchDir(path, err) => {
                write!(f, "failed to watch directory {}: {}", path.to_string_lossy(), err)
            },
            Self::TokioRuntime(err) => {
                write!(f, "failed to create async runtime: {}", err)
            },
            Self::Bind(addr, err) => {
                write!(f, "failed to bind {}: {}", addr, err)
            },
            Self::Server(err) => {
                write!(f, "error while running server: {}", err)
            },
        }
    }
}

impl error::Error for Error {}
