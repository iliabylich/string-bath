/// A sum type of all error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StringPoolError {
    /// Given string is too long for a string pool.
    StringIsTooLong,

    /// No space in the pool.
    NoSpaceInPool,
}

impl core::fmt::Display for StringPoolError {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::StringIsTooLong => f.write_str("StringIsTooLong"),
            Self::NoSpaceInPool => f.write_str("NoSpaceInPool"),
        }
    }
}

impl core::error::Error for StringPoolError {}
