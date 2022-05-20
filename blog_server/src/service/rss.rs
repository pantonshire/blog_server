use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::Extension,
};

use super::response::Rss;
use crate::{posts_store::ConcurrentPostsStore, Config};

pub async fn handle(
    Extension(config): Extension<Arc<Config>>,
    Extension(posts): Extension<ConcurrentPostsStore>,
) -> Rss<Bytes> {
    let rss_items = posts.read()
        .await
        .iter_by_created()
        .take(config.rss.num_posts)
        .map(|post| {
            rss::ItemBuilder::default()
                .title(Some(post.title().to_owned()))
                .link(Some(format!(
                    "{}://{}/articles/{}",
                    config.rss.protocol,
                    config.rss.domain,
                    post.id()
                )))
                .pub_date(Some(post.created().to_rfc2822()))
                .build()
        })
        .collect::<Vec<rss::Item>>();

    Rss(rss::ChannelBuilder::default()
        .title(config.rss.title.clone())
        .link(format!(
            "{}://{}",
            config.rss.protocol, config.rss.domain
        ))
        .ttl(Some(config.rss.ttl.to_string()))
        .items(rss_items)
        .build()
        .to_string()
        .into())
}
