use crate::StringPoolError;
use core::cell::Cell;

/// Representation of a slot in a string pool.
#[derive(Debug)]
#[repr(C)]
pub(crate) struct Slot<const LEN: usize> {
    pub(crate) str: Cell<[u8; LEN]>,
    pub(crate) force_zero: u8,
    pub(crate) len: Cell<usize>,
    pub(crate) refcount: Cell<usize>,
}

impl<const LEN: usize> Slot<LEN> {
    /// Constructs an empty slot.
    pub(crate) const fn new_empty() -> Self {
        Self {
            str: Cell::new([0; LEN]),
            force_zero: 0,
            len: Cell::new(0),
            refcount: Cell::new(0),
        }
    }

    /// Acquires a slot and fills it with a given string.
    ///
    /// # Errors
    ///
    /// Returns an error if the given string doesn't fit into a slot.
    pub(crate) fn acquire(&self, str: &str) -> Result<(), StringPoolError> {
        let src = str.as_bytes();
        if src.len() > LEN {
            return Err(StringPoolError::StringIsTooLong);
        }
        let len = core::cmp::min(src.len(), LEN);

        let mut dst = [0; LEN];
        dst.get_mut(0..len)
            .ok_or(StringPoolError::StringIsTooLong)?
            .copy_from_slice(src);

        self.str.set(dst);
        self.len.set(len);
        self.refcount.set(1);

        Ok(())
    }

    /// Resets a slot so that a pool that owns it may re-use it.
    pub(crate) fn release(&self) {
        self.str.set([0; _]);
        self.len.set(0);
        self.refcount.set(0);
    }

    /// Returns a byte representation of a slot.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        // SAFETY: once the `Slot` is acquired the data inside it is frozen.
        //         Nobody mutates `self.str` or `self.len`, even the `Slot` itself.
        //         The only exception is `Slot::release()` but it is only called by `Drop` on a `StringRef`
        //         if the reference is the last one (which is tracked using `refcount` field).
        let str = unsafe { &*self.str.as_ptr() };

        // SAFETY: empty slot has `len=0` so it's always safe to `.get()` it.
        //         occupied slot can only be constructed by `.acquire()` method which guarantees validity of the data.
        unsafe { str.get_unchecked(..self.len.get()) }
    }

    /// Returns a string representation of a slot.
    pub(crate) fn as_str(&self) -> &str {
        // SAFETY: `self.as_bytes()` is either an empty slice or a valid UTF-8 string because it was
        //         constructed based on a valid `&str`.
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }

    pub(crate) fn inc_refcount(&self) {
        self.refcount.update(|count| count.wrapping_add(1));
    }

    pub(crate) fn dec_refcount(&self) {
        self.refcount.update(|count| count.wrapping_sub(1));
    }

    pub(crate) const fn is_free(&self) -> bool {
        self.refcount.get() == 0
    }
}
