use std::sync::Arc;

use atom_syndication as atom;
use axum::{body::Bytes, extract::Extension};

use blog::time::unix_epoch;

use crate::Context;

use super::response::Atom;

pub(super) async fn handle(Extension(context): Extension<Arc<Context>>) -> Atom<Bytes> {
    let (atom_entries, updated) = {
        let guard = context.posts().read().await;

        let atom_entries = guard
            .iter_by_published()
            .take(context.config().atom.num_posts)
            .map(|post| {
                atom::EntryBuilder::default()
                    .id(format!("urn:uuid:{}", post.uuid()))
                    .title(post.title().to_owned())
                    .updated(post.updated())
                    .links(vec![atom::LinkBuilder::default()
                        .href(format!(
                            "{}://{}/articles/{}",
                            context.config().site.protocol,
                            context.config().site.domain,
                            post.id()
                        ))
                        .rel("alternate".to_owned())
                        .mime_type(Some("text/html".to_owned()))
                        .build()])
                    .author(
                        atom::PersonBuilder::default()
                            .name(post.author().to_owned())
                            .build(),
                    )
                    .build()
            })
            .collect::<Vec<atom::Entry>>();

        let updated = guard.last_updated().unwrap_or_else(unix_epoch);

        (atom_entries, updated)
    };

    Atom(
        atom::FeedBuilder::default()
            .id(format!("urn:uuid:{}", context.config().namespace_uuid))
            .title(context.config().atom.title.clone())
            .updated(updated)
            .links(vec![
                atom::LinkBuilder::default()
                    .href(format!(
                        "{}://{}/atom.xml",
                        context.config().site.protocol, context.config().site.domain
                    ))
                    .rel("self".to_owned())
                    .build(),
                atom::LinkBuilder::default()
                    .href(format!(
                        "{}://{}/articles/",
                        context.config().site.protocol, context.config().site.domain
                    ))
                    .rel("alternate".to_owned())
                    .mime_type(Some("text/html".to_owned()))
                    .build(),
            ])
            .entries(atom_entries)
            .build()
            .to_string()
            .into(),
    )
}
