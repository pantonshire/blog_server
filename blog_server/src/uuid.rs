use std::ops;

use knuffel::{
    ast::{Literal, TypeName},
    decode::{Context, Kind},
    errors::{DecodeError, ExpectedType},
    span::Spanned,
    traits::ErrorSpan,
    DecodeScalar,
};

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
