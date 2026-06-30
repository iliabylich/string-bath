/// A sum type of all error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StringPoolError {
    /// Given string is too long for a string pool.
    StringIsTooLong {
        /// Maximum length that a pool accepts.
        max_length: usize,
        /// Length of a given string.
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
