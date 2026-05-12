/// A sum type of all error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StringPoolError {
    /// Given string is too long for a string pool.
    StringIsTooLong {
        /// Length of a given string.
        max_length: usize,
        /// Maximum length that a pool accepts.
        actual_length: usize,
    },

    /// No space in the pool.
    NoSpaceInPool,
}

impl core::fmt::Display for StringPoolError {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::StringIsTooLong {
                max_length,
                actual_length,
            } => write!(f, "StringIsTooLong({actual_length}/{max_length})"),
            Self::NoSpaceInPool => write!(f, "NoSpaceInPool"),
        }
    }
}

impl core::error::Error for StringPoolError {}
