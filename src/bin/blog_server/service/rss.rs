use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::Extension,
};

use blog::{
    db::ConcurrentPostsStore,
    time::unix_epoch,
};

use crate::Config;

use super::response::Rss;

pub(super) async fn handle(
    Extension(config): Extension<Arc<Config>>,
    Extension(posts): Extension<ConcurrentPostsStore>,
) -> Rss<Bytes> {
    let (rss_items, updated) = {
        let guard = posts.read().await;

        let rss_items = guard.iter_by_published()
            .take(config.rss.num_posts)
            .map(|post| {
                rss::ItemBuilder::default()
                    .title(Some(post.title().to_owned()))
                    .guid(Some(rss::GuidBuilder::default()
                        .value(post.uuid().to_string())
                        .permalink(false)
                        .build()))
                    .link(Some(format!(
                        "{}://{}/articles/{}",
                        config.self_ref.protocol,
                        config.self_ref.domain,
                        post.id()
                    )))
                    .pub_date(Some(post.published().to_rfc2822()))
                    .build()
            })
            .collect::<Vec<rss::Item>>();

        let updated = guard.last_updated()
            .unwrap_or_else(unix_epoch);

        (rss_items, updated)
    };

    Rss(rss::ChannelBuilder::default()
        .title(config.rss.title.clone())
        .link(format!(
            "{}://{}",
            config.self_ref.protocol, config.self_ref.domain
        ))
        .ttl(Some(config.rss.ttl.to_string()))
        .last_build_date(Some(updated.to_rfc2822()))
        .items(rss_items)
        .build()
        .to_string()
        .into())
}
