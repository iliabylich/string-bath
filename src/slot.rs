use crate::StringPoolError;
use core::cell::Cell;

/// Representation of a slot in a string pool.
#[derive(Debug)]
#[repr(C)]
pub(crate) struct Slot<const LEN: usize> {
    pub(crate) str: [u8; LEN],
    pub(crate) force_zero: u8,
    pub(crate) len: usize,
    pub(crate) refcount: Cell<u8>,
    pub(crate) free: bool,
}

impl<const LEN: usize> Slot<LEN> {
    /// Constructs an empty slot.
    pub(crate) const fn new_empty() -> Self {
        Self {
            str: [0; LEN],
            force_zero: 0,
            len: 0,
            refcount: Cell::new(0),
            free: true,
        }
    }

    /// Acquires a slot and fills it with a given string.
    ///
    /// # Errors
    ///
    /// Returns an error if the given string doesn't fit into a slot.
    pub(crate) fn acquire(&mut self, str: &str) -> Result<(), StringPoolError> {
        if LEN == 0 || str.len() > LEN {
            return Err(StringPoolError::StringIsTooLong {
                max_length: LEN,
                actual_length: str.len(),
            });
        }

        let from = str.as_bytes();
        // SAFETY: we validated length of the `str` above, it fits into a slot.
        let to = unsafe { self.str.get_unchecked_mut(0..from.len()) };

        to.copy_from_slice(from);

        self.len = str.len();
        self.free = false;
        self.refcount = Cell::new(1);

        Ok(())
    }

    /// Resets a slot so that a pool that owns it may re-use it.
    pub(crate) const fn release(&mut self) {
        self.str = [0; LEN];
        self.len = 0;
        self.refcount = Cell::new(0);
        self.free = true;
    }

    /// Returns a byte representation of a slot.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        // SAFETY: both empty and occupied slots are constructed by a `StringPool` and so:
        //         1. `self.str[...self.len]` is always initialized
        unsafe { self.str.get_unchecked(..self.len) }
    }

    /// Returns a string representation of a slot.
    pub(crate) fn as_str(&self) -> &str {
        // SAFETY: both empty and occupied slots are constructed by a `StringPool` and so:
        //         1. `self.str[...self.len]` is always initialized
        //         2. `self.str[...self.len]` is a valid UTF-8 string
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }

    /// Increments reference count.
    pub(crate) fn inc_refcount(&self) {
        self.refcount.update(|count|
                // SAFETY: for an overflow to happen there must be 2^64 `.clone()` calls
                //         which is unrealistic.
                unsafe { count.unchecked_add(1) });
    }

    /// Decrements reference count.
    pub(crate) fn dec_refcount(&self) {
        self.refcount.update(|count|
                // SAFETY: `refcount` is incremented on allocation and copying
                //          and is decremented in `Drop`, so we always have capacity here.
                unsafe { count.unchecked_sub(1) });
    }
}
