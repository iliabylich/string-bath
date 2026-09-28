use core::{cell::Cell, ffi::CStr};

/// Representation of a slot in a string pool.
#[derive(Debug)]
#[repr(C)]
pub(crate) struct Slot<const LEN: usize> {
    buf: BufWithGuaranteedNul<LEN>,
    len: Cell<usize>,
    refcount: Cell<usize>,
}

#[derive(Debug)]
#[repr(C)]
struct BufWithGuaranteedNul<const N: usize> {
    pub(crate) bytes: Cell<[u8; N]>,
    nul: u8,
}

impl<const LEN: usize> Slot<LEN> {
    pub(crate) const fn new_empty() -> Self {
        Self {
            buf: BufWithGuaranteedNul {
                bytes: Cell::new([0; LEN]),
                nul: 0,
            },
            len: Cell::new(0),
            refcount: Cell::new(0),
        }
    }

    // SAFETY: it's safe to call this function if:
    //         1. `str` fits into a slot
    //         2. `str` doesn't have a NUL byte
    pub(crate) unsafe fn acquire(&self, str: &str) {
        let src = str.as_bytes();
        let len = src.len();

        let dst = self.buf.bytes.as_ptr().cast::<u8>();
        // SAFETY: the string fits into a slot, and we are the only holder of this `Slot`,
        //         so copying `len` bytes into the slot is safe.
        unsafe {
            core::ptr::copy(src.as_ptr(), dst, len);
        };
        if len < LEN {
            // SAFETY: `len` is less than `LEN`, so the address at `dst+len` is within the buffer range
            let null_dst = unsafe { dst.add(len) };

            // SAFETY: `len` is less than `LEN`, so the address at `dst+len` is within the buffer range
            //         and writable
            unsafe {
                null_dst.write(0);
            };
        }

        self.len.set(len);
        self.refcount.set(1);
    }

    pub(crate) fn release(&self) {
        self.len.set(0);
        self.refcount.set(0);
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        // SAFETY: once the `Slot` is acquired the data inside it is frozen.
        //         Nobody mutates `self.str` or `self.len`, even the `Slot` itself.
        //         The only exception is `Slot::release()` but it is only called by `Drop` on the last `StringRef`.
        let str = unsafe { &*self.buf.bytes.as_ptr() };

        // SAFETY: empty slot has `len=0` so it's always safe to `get()` it.
        //         occupied slot can only be constructed by `acquire()` method which guarantees validity of the data.
        unsafe { str.get_unchecked(..self.len.get()) }
    }

    pub(crate) fn as_str(&self) -> &str {
        // SAFETY: `self.as_bytes()` is either an empty slice or a valid UTF-8 string because it was
        //         constructed based on a valid `&str`.
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }

    pub(crate) const fn as_c_str(&self) -> &CStr {
        let ptr = (&raw const self.buf).cast();
        // SAFETY: `buf` is always filled with a valid initialized sequence of bytes with a trailing NUL
        //         because the only way to initialize a `Slot` is to call `acquire()` that guarantees
        //         validity of the data.
        unsafe { CStr::from_ptr(ptr) }
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
    pub(crate) fn dec_refcount(&self) {
        self.refcount.update(|count| {
            count
                .checked_sub(1)
                .unwrap_or_else(|| panic!("Slot refcount underflow"))
        });
    }

    #[must_use]
    pub(crate) const fn refcount(&self) -> usize {
        self.refcount.get()
    }

    pub(crate) const fn is_free(&self) -> bool {
        self.refcount.get() == 0
    }
}
