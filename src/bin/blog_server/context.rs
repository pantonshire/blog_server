use blog::db::ConcurrentPostsStore;

use crate::Config;

pub(crate) struct Context {
    config: Config,
    posts: ConcurrentPostsStore,
}

impl Context {
    #[inline]
    #[must_use]
    pub(crate) fn new(config: Config, posts: ConcurrentPostsStore) -> Self {
        Self {
            config,
            posts,
        }
    }

    #[inline]
    #[must_use]
    pub(crate) fn config(&self) -> &Config {
        &self.config
    }

    #[inline]
    #[must_use]
    pub(crate) fn posts(&self) -> &ConcurrentPostsStore {
        &self.posts
    }
}
