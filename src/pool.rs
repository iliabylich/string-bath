use core::cell::UnsafeCell;

use crate::{StringPoolError, StringRef, slot::Slot};

/// A string pool containing `SLOTS_COUNT` slots, each can store up to `STRING_LEN` bytes.
#[derive(Debug)]
pub struct StringPool<const SLOTS_COUNT: usize, const STRING_LEN: usize> {
    pub(crate) slots: [UnsafeCell<Slot<STRING_LEN>>; SLOTS_COUNT],
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
            slots: [const { UnsafeCell::new(Slot::new_empty()) }; _],
        }
    }

    /// Copies a given string to one of the pool's slots and returns a pointer to this slot.
    ///
    /// # Errors
    ///
    /// Returns an error if the pool is empty.
    #[inline]
    pub fn alloc(&self, str: &str) -> Result<StringRef<'_, STRING_LEN>, StringPoolError> {
        for slot in &self.slots {
            let ptr = UnsafeCell::raw_get(slot);
            // SAFETY: `ptr` comes from an `UnsafeCell`, so it is
            //         valid, aligned, initialized, and lives for the duration of this borrow.
            //         Reading `free` does not create a reference to the slot so it doesn't
            //         conflict with other existing references to occupied slots.
            if unsafe { (*ptr).free } {
                // SAFETY: there are no other **references** to the inner value of the **free** slot,
                //         so it is safe to create one.
                let inner = unsafe { &mut *ptr };
                inner.acquire(str)?;
                return Ok(StringRef {
                    slot: inner,
                    _phantom: core::marker::PhantomData,
                });
            }
        }

        Err(StringPoolError::NoSpaceInPool)
    }
}
