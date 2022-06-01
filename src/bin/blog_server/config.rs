use std::{path::PathBuf, ops};

use knuffel::{
    ast::{Literal, TypeName},
    decode::{Context, Kind},
    errors::{DecodeError, ExpectedType},
    span::Spanned,
    traits::ErrorSpan,
    DecodeScalar,
};

#[derive(knuffel::Decode, Clone, Debug)]
pub struct Config {
    #[knuffel(child, unwrap(argument))]
    pub bind: String,
    #[knuffel(child, unwrap(argument))]
    pub concurrency_limit: usize,
    #[knuffel(child, unwrap(argument))]
    pub static_dir: PathBuf,
    #[knuffel(child, unwrap(argument))]
    pub favicon_dir: PathBuf,
    #[knuffel(child, unwrap(argument))]
    pub robots_path: PathBuf,
    #[knuffel(child, unwrap(argument))]
    pub posts_dir: PathBuf,
    #[knuffel(child, unwrap(argument))]
    pub post_media_dir: PathBuf,
    #[knuffel(child, unwrap(argument))]
    pub namespace_uuid: Uuid,
    #[knuffel(child)]
    pub self_ref: SelfRefConfig,
    #[knuffel(child)]
    pub rss: RssConfig,
    #[knuffel(child)]
    pub atom: AtomConfig,
}

#[derive(knuffel::Decode, Clone, Debug)]
pub struct SelfRefConfig {
    #[knuffel(child, unwrap(argument))]
    pub protocol: String,
    #[knuffel(child, unwrap(argument))]
    pub domain: String,
}

#[derive(knuffel::Decode, Clone, Debug)]
pub struct RssConfig {
    #[knuffel(child, unwrap(argument))]
    pub num_posts: usize,
    #[knuffel(child, unwrap(argument))]
    pub title: String,
    #[knuffel(child, unwrap(argument))]
    pub ttl: u32,
}

#[derive(knuffel::Decode, Clone, Debug)]
pub struct AtomConfig {
    #[knuffel(child, unwrap(argument))]
    pub num_posts: usize,
    #[knuffel(child, unwrap(argument))]
    pub title: String,
}

#[derive(Clone, Copy, Default, Debug)]
#[repr(transparent)]
pub struct Uuid(pub libshire::uuid::Uuid);

impl Uuid {
    pub fn as_inner(&self) -> &libshire::uuid::Uuid {
        &self.0
    }
}

impl ops::Deref for Uuid {
    type Target = libshire::uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        self.as_inner()
    }
}

impl<S: ErrorSpan> DecodeScalar<S> for Uuid {
    fn type_check(type_name: &Option<Spanned<TypeName, S>>, ctx: &mut Context<S>) {
        if let Some(type_name) = type_name {
            ctx.emit_error(DecodeError::TypeName {
                span: type_name.span().clone(),
                found: Some((&**type_name).clone()),
                expected: ExpectedType::no_type(),
                rust_type: "Uuid",
            });
        }
    }

    fn raw_decode(
        value: &Spanned<Literal, S>,
        ctx: &mut Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        match &**value {
            Literal::String(s) => match s.parse() {
                Ok(uuid) => Ok(Self(uuid)),
                Err(err) => {
                    ctx.emit_error(DecodeError::conversion(value, err));
                    Ok(Default::default())
                }
            },
            _ => {
                ctx.emit_error(DecodeError::scalar_kind(Kind::String, value));
                Ok(Default::default())
            }
        }
    }
}
