use std::{
    collections::{BTreeSet, hash_map, HashMap, HashSet},
    iter::FusedIterator,
    sync::Arc,
};

use chrono::{DateTime, Utc};
use libshire::strings::ShString22;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::post::{Post, Id};

#[derive(Clone)]
pub struct ConcurrentPostsStore {
    inner: Arc<RwLock<PostsStore>>,
}

impl ConcurrentPostsStore {
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(PostsStore::new())) }
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, PostsStore> {
        self.inner.read().await
    }

    pub fn write_blocking(&self) -> RwLockWriteGuard<'_, PostsStore> {
        self.inner.blocking_write()
    }

    pub async fn get(&self, id: &str) -> Option<Arc<Post>> {
        self.read().await.get(id).cloned()
    }
}

impl Default for ConcurrentPostsStore {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PostsStore {
    posts: HashMap<Id, Arc<Post>>,
    published_ix: BTreeSet<PublishedIxEntry>,
    tags_ix: HashMap<ShString22, HashSet<Id>>,
}

// TODO: shrink the various collections on removal to deallocate unneeded space

impl PostsStore {
    pub fn new() -> Self {
        Self {
            posts: HashMap::new(),
            published_ix: BTreeSet::new(),
            tags_ix: HashMap::new(),
        }
    }

    pub fn get(&self, id: &str) -> Option<&Arc<Post>> {
        self.posts.get(id)
    }

    pub fn insert(&mut self, post: Post) -> Option<Arc<Post>> {
        let old_post = self.remove(post.id());

        // Insert the post into each of the tag indexes.
        for tag in post.tags() {
            // First, get the existing `HashSet` for the tag, or create a new one if one does not
            // already exist. Then, insert the post's ID into the `HashSet`.
            match self.tags_ix.entry(tag.clone()) {
                hash_map::Entry::Occupied(entry) => entry.into_mut(),
                hash_map::Entry::Vacant(entry) => entry.insert(HashSet::new()),
            }.insert(post.id().clone());
        }

        // Insert the post into the correct position of the published BTree index.
        self.published_ix.insert(PublishedIxEntry::new(&post));

        // Wrap the post with an atomic reference counter and insert it into the main posts
        // `HashMap`.
        self.posts.insert(post.id().clone(), Arc::new(post));

        old_post
    }

    pub fn remove(&mut self, id: &str) -> Option<Arc<Post>> {
        match self.posts.remove(id) {
            Some(post) => {
                // Remove the post's entry in the published index.
                self.published_ix
                    .remove(&PublishedIxEntry::new(&post));

                // Remove every occurence of the post from the tags index.
                for tag in post.tags() {
                    if let Some(tag_ix) = self.tags_ix.get_mut(tag) {
                        tag_ix.remove(id);
                    }
                }

                Some(post)
            },
            None => None,
        }
    }

    pub fn clear(&mut self) {
        self.tags_ix.clear();
        self.published_ix.clear();
        self.posts.clear();
    }

    pub fn last_updated(&self) -> Option<DateTime<Utc>> {
        self.iter().map(|post| post.updated()).max()
    }

    pub fn iter(&self)
    -> impl '_
        + Iterator<Item = &Arc<Post>>
        + ExactSizeIterator
        + FusedIterator
        + Clone
    {
        self.posts.values()
    }

    pub fn iter_by_published(&self)
    -> impl '_
        + Iterator<Item = &Arc<Post>>
        + DoubleEndedIterator
        + ExactSizeIterator
        + FusedIterator
        + Clone
    {
        // For each entry of the published index, look up the corresponding post in the posts map
        // and return the post. Every entry in the published index should contain the ID of a post
        // in the posts map, so the `expect` should never fail.
        self.published_ix
            .iter()
            .map(|entry| self.get(&entry.id)
                .expect("invalid entry in `published_ix` pointing to a post that does not exist"))
    }
}

impl Default for PostsStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct PublishedIxEntry {
    published: DateTime<Utc>,
    id: Id,
}

impl PublishedIxEntry {
    fn new(post: &Post) -> Self {
        Self {
            published: post.published(),
            id: post.id().clone(),
        }
    }
}
