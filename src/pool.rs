use crate::{StringPoolError, StringRef, slot::Slot};

/// A string pool containing `SLOTS_COUNT` slots, each can store up to `STRING_LEN` bytes.
#[derive(Debug)]
pub struct StringPool<const SLOTS_COUNT: usize, const STRING_LEN: usize> {
    pub(crate) slots: [Slot<STRING_LEN>; SLOTS_COUNT],
}

impl<const SLOTS_COUNT: usize, const STRING_LEN: usize> Default
    for StringPool<SLOTS_COUNT, STRING_LEN>
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const SLOTS_COUNT: usize, const STRING_LEN: usize> StringPool<SLOTS_COUNT, STRING_LEN> {
    /// Constructs a new string pool.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: [const { Slot::new_empty() }; _],
        }
    }

    /// Copies a given string to one of the pool's slots and returns a pointer to this slot.
    ///
    /// # Errors
    ///
    /// Returns an error if the pool is full.
    #[inline]
    pub fn alloc(&self, str: &str) -> Result<StringRef<'_, STRING_LEN>, StringPoolError> {
        if str.as_bytes().contains(&0) {
            return Err(StringPoolError::StringContainsNulByte);
        }
        if str.len() > STRING_LEN {
            return Err(StringPoolError::StringIsTooLong);
        }

        let slot = self
            .slots
            .iter()
            .find(|slot| slot.is_free())
            .ok_or(StringPoolError::NoSpaceInPool)?;

        slot.acquire(str)?;
        Ok(StringRef { slot })
    }
}
