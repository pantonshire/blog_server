use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    NoDelim,
    Header(Box<toml::de::Error>),
    IdTooLong(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDelim => {
                write!(f, "post has no header; no delimiter `\\n---\\n` found")
            },
            Self::Header(err) => {
                write!(f, "error decoding post header: {}", err)
            },
            Self::IdTooLong(len) => {
                write!(f, "post id too long: {} bytes", len)
            },
        }
    }
}

impl error::Error for Error {}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Self::Header(Box::new(err))
    }
}
