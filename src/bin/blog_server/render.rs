use std::{
    fmt,
    fs,
    io::{self, Read},
    path::PathBuf,
    sync::{Arc, mpsc},
};

use chrono::DateTime;
use notify::DebouncedEvent;
use tracing::{info, warn, error};

use blog::{
    codeblock::CodeBlockRenderer,
    post::{Error as ParseError, Post, Id},
    db::ConcurrentPostsStore,
};

use crate::Config;

pub(crate) struct Renderer {
    config: Arc<Config>,
    posts: ConcurrentPostsStore,
    code_renderer: CodeBlockRenderer,
    posts_dir_path: PathBuf,
    rx: mpsc::Receiver<DebouncedEvent>,
}

impl Renderer {
    pub(crate) fn new(
        config: Arc<Config>,
        posts: ConcurrentPostsStore,
        code_renderer: CodeBlockRenderer,
        posts_dir_path: PathBuf,
    ) -> (Self, mpsc::Sender<DebouncedEvent>)
    {
        let (tx, rx) = mpsc::channel();

        // Buffer a rescan event here so that it will be the first event received when
        // `handle_events` is called. This will cause the `Renderer` to perform an "initial scan"
        // of the post files.
        tx.send(DebouncedEvent::Rescan).unwrap();

        (Self {
            config,
            posts,
            code_renderer,
            posts_dir_path,
            rx,
        }, tx)
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn handle_events(self) {
        while let Ok(notify_event) = self.rx.recv() {
            let fs_event = match notify_event {
                // Convert create & write events for valid post file names to update events.
                DebouncedEvent::Create(path) | DebouncedEvent::Write(path) => {
                    EventTarget::from_path(path)
                        .map(Event::Update)
                },
    
                // Convert remove events for valid post file names.
                DebouncedEvent::Remove(path) => {
                    EventTarget::from_path(path)
                        .map(Event::Remove)
                },
    
                // Convert rename events depending on whether the old / new paths are valid post
                // file names.
                DebouncedEvent::Rename(old_path, new_path) => {
                    match (EventTarget::from_path(old_path), EventTarget::from_path(new_path)) {
                        (Some(old_target), Some(new_target)) => Some(Event::Rename(old_target, new_target)),
                        (None, Some(new_target)) => Some(Event::Update(new_target)),
                        (Some(old_target), None) => Some(Event::Remove(old_target)),
                        (None, None) => None,
                    }
                },
    
                // Convert rescan events, where it is necessary to read the directory's contents.
                DebouncedEvent::Rescan => Some(Event::Scan),
    
                // Ignore all other events.
                _ => None,
            };
    
            if let Some(fs_event) = fs_event {
                self.handle_event(&fs_event);
            }
        }

        info!("Filesystem events channel closed, exiting");
    }

    fn handle_event(&self, event: &Event) {
        info!(event = ?event);
        match event {
            Event::Update(target) => self.update(target),
            Event::Rename(old_target, new_target) => self.rename(old_target, new_target),
            Event::Remove(target) => self.remove(target),
            Event::Scan => self.scan(),
        }
    }

    #[tracing::instrument(skip(self))]
    fn update(&self, target: &EventTarget) {
        match self.parse_post_from_target(target) {
            Ok(post) => {
                let mut guard = self.posts.write_blocking();
                guard.insert(post);
            },
            Err(err) => {
                err.log();
            }
        };
    }

    #[tracing::instrument(skip(self))]
    fn rename(&self, old_target: &EventTarget, new_target: &EventTarget) {
        let post_res = self.parse_post_from_target(new_target);
        let mut guard = self.posts.write_blocking();
        guard.remove(&old_target.id);
        match post_res {
            Ok(post) => {
                guard.insert(post);
            },
            Err(err) => {
                err.log();
            },
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove(&self, target: &EventTarget) {
        let mut guard = self.posts.write_blocking();
        guard.remove(&target.id);
    }

    #[tracing::instrument(skip(self))]
    fn scan(&self) {
        let posts_dir = match fs::read_dir(&self.posts_dir_path) {
            Ok(posts_dir) => posts_dir,
            Err(err) => {
                Error::Io(Box::new(err)).log();
                return;
            },
        };

        let mut posts = Vec::new();

        for dir_entry in posts_dir {
            let dir_entry = match dir_entry {
                Ok(dir_entry) => dir_entry,
                Err(err) => {
                    Error::Io(Box::new(err)).log();
                    continue;
                },
            };

            if let Some(target) = EventTarget::from_path(dir_entry.path()) {
                posts.push(match self.parse_post_from_target(&target) {
                    Ok(post) => post,
                    Err(err) => {
                        err.log();
                        continue;
                    },
                });
            }
        }
        
        let mut guard = self.posts.write_blocking();
        guard.clear();
        for post in posts {
            guard.insert(post);
        }
    }

    fn parse_post_from_target(&self, target: &EventTarget) -> Result<Post, Error> {
        let mut fd = fs::OpenOptions::new()
            .read(true)
            .open(&target.path)
            .map_err(|err| Error::Io(Box::new(err)))?;
    
        let metadata = fd.metadata()
            .map_err(|err| Error::Io(Box::new(err)))?;
    
        if !metadata.file_type().is_file() {
            return Err(Error::NotAFile);
        }
    
        let updated = metadata
            .modified()
            .ok()
            .map(DateTime::from);
    
        let contents = {
            let mut buf = String::new();
            fd.read_to_string(&mut buf)
                .map_err(|err| Error::Io(Box::new(err)))?;
            buf
        };
    
        drop(fd);
    
        Post::new_from_str(
            &self.code_renderer,
            self.config.namespace_uuid,
            target.id.clone(),
            updated,
            &contents
        ).map_err(|err| Error::Parsing(Box::new(err)))
    }
}

#[derive(Debug)]
enum Event {
    Update(EventTarget),
    Rename(EventTarget, EventTarget),
    Remove(EventTarget),
    Scan,
}

struct EventTarget {
    path: PathBuf,
    id: Id,
}

impl fmt::Debug for EventTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.id, self.path.to_string_lossy())
    }
}

impl EventTarget {
    fn from_path(path: PathBuf) -> Option<Self> {
        path.file_name()
            .and_then(|file_name| file_name.to_str())
            .and_then(Id::from_file_name)
            .map(|id| Self {
                path,
                id,
            })
    }
}

pub(crate) enum Error {
    Io(Box<io::Error>),
    NotAFile,
    Parsing(Box<ParseError>),
}

impl Error {
    fn log(&self) {
        match self {
            Error::Io(err) => {
                error!(error = %err, "IO error while processing event");
            },
            Error::NotAFile => {
                warn!("Event target is not a regular file");
            },
            Error::Parsing(err) => {
                warn!(error = %err, "Parsing error while processing event");
            },
        }
    }
}
