use crate::StringPoolError;
use core::{cell::Cell, ffi::c_char};

/// Representation of a slot in a string pool.
#[derive(Debug)]
#[repr(C)]
pub(crate) struct Slot<const LEN: usize> {
    buf: Buf<LEN>,
    len: Cell<usize>,
    refcount: Cell<usize>,
}

#[derive(Debug)]
#[repr(C)]
struct Buf<const N: usize> {
    pub(crate) bytes: Cell<[u8; N]>,
    nul: u8,
}

impl<const LEN: usize> Slot<LEN> {
    pub(crate) const fn new_empty() -> Self {
        Self {
            buf: Buf {
                bytes: Cell::new([0; LEN]),
                nul: 0,
            },
            len: Cell::new(0),
            refcount: Cell::new(0),
        }
    }

    /// Acquires a slot and fills it with a given string.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// 1. given string doesn't fit into a slot
    /// 2. given string contains a NUL byte
    /// 3. the pool is full
    pub(crate) fn acquire(&self, str: &str) -> Result<(), StringPoolError> {
        let src = str.as_bytes();
        let len = src.len();

        let mut dst = [0; LEN];
        dst.get_mut(0..len)
            .ok_or(StringPoolError::StringIsTooLong)?
            .copy_from_slice(src);

        self.buf.bytes.set(dst);
        self.len.set(len);
        self.refcount.set(1);

        Ok(())
    }

    pub(crate) fn release(&self) {
        self.buf.bytes.set([0; _]);
        self.len.set(0);
        self.refcount.set(0);
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        // SAFETY: once the `Slot` is acquired the data inside it is frozen.
        //         Nobody mutates `self.str` or `self.len`, even the `Slot` itself.
        //         The only exception is `Slot::release()` but it is only called by `Drop` on a `StringRef`
        //         if the reference is the last one (which is tracked using `refcount` field).
        let str = unsafe { &*self.buf.bytes.as_ptr() };

        // SAFETY: empty slot has `len=0` so it's always safe to `.get()` it.
        //         occupied slot can only be constructed by `.acquire()` method which guarantees validity of the data.
        unsafe { str.get_unchecked(..self.len.get()) }
    }

    pub(crate) fn as_str(&self) -> &str {
        // SAFETY: `self.as_bytes()` is either an empty slice or a valid UTF-8 string because it was
        //         constructed based on a valid `&str`.
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }

    pub(crate) const fn as_ptr(&self) -> *const c_char {
        (&raw const self.buf).cast()
    }

    #[expect(clippy::panic)]
    pub(crate) fn inc_refcount(&self) {
        self.refcount.update(|count| {
            count
                .checked_add(1)
                .unwrap_or_else(|| panic!("Slot refcount overflow"))
        });
    }

    #[expect(clippy::panic)]
    #[must_use]
    pub(crate) fn dec_refcount(&self) -> usize {
        self.refcount.update(|count| {
            count
                .checked_sub(1)
                .unwrap_or_else(|| panic!("Slot refcount underflow"))
        });
        self.refcount.get()
    }

    pub(crate) const fn is_free(&self) -> bool {
        self.refcount.get() == 0
    }

    #[cfg(test)]
    pub(crate) fn raw_bytes(&self) -> [u8; LEN] {
        self.buf.bytes.get()
    }
}
