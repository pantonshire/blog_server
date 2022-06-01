use std::{error, fmt};

use kdl::KdlError;

#[derive(Debug)]
pub enum Error {
    NoDelim,
    Syntax(Box<KdlError>),
    FieldMissing {
        field: &'static str,
    },
    BadType {
        field: &'static str,
        expected: &'static str,
    },
    IdTooLong(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDelim => {
                write!(f, "post has no header; no delimiter `\\n---\\n` found")
            },
            Self::Syntax(err) => {
                write!(f, "syntax error in post header: {}", err)
            },
            Self::FieldMissing { field } => {
                write!(f, "missing required post header field `{}`", field)
            },
            Self::BadType { field, expected } => {
                write!(f, "expected post header field `{}` to be {}", field, expected)
            },
            Self::IdTooLong(len) => {
                write!(f, "post id too long: {} bytes", len)
            },
        }
    }
}

impl error::Error for Error {}

impl From<KdlError> for Error {
    fn from(err: KdlError) -> Self {
        Self::Syntax(Box::new(err))
    }
}
