use std::{borrow, fmt, ops};

use libshire::strings::ShString22;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Id(ShString22);

impl Id {
    #[inline]
    #[must_use]
    pub fn from_file_name(file_name: &str) -> Option<Self> {
        fn is_invalid_char(c: char) -> bool {
            c == '/' || c == '\\' || c == '.'
        }

        let prefix = file_name
            .strip_suffix(super::POST_FILE_EXTENSION)?;

        if prefix.contains(is_invalid_char) {
            return None;
        }

        Some(Self(ShString22::new_from_str(prefix)))
    }

    #[inline]
    #[must_use]
    pub fn into_inner(self) -> ShString22 {
        self.0
    }

    #[inline]
    #[must_use]
    pub fn as_inner(&self) -> &ShString22 {
        &self.0
    }
}

impl ops::Deref for Id {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl ops::DerefMut for Id {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl AsRef<str> for Id {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsMut<str> for Id {
    #[inline]
    fn as_mut(&mut self) -> &mut str {
        self
    }
}

impl borrow::Borrow<str> for Id {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

impl borrow::BorrowMut<str> for Id {
    #[inline]
    fn borrow_mut(&mut self) -> &mut str {
        self
    }
}

impl fmt::Display for Id {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
