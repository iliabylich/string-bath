use crate::slot::Slot;

/// A pool-allocated string.
pub struct StringRef<'pool, const N: usize> {
    pub(crate) slot: *mut Slot<N>,
    pub(crate) _phantom: core::marker::PhantomData<&'pool ()>,
}

impl<'pool, const N: usize> StringRef<'pool, N> {
    const fn slot(&self) -> &'pool Slot<N> {
        // SAFETY: `self.slot` lives for `'pool` so it's safe to dereference it.
        unsafe { &*self.slot }
    }

    const fn slot_mut(&mut self) -> &'pool mut Slot<N> {
        // SAFETY: `self.slot` lives for `'pool` so it's safe to dereference it.
        unsafe { &mut *self.slot }
    }

    /// Converts `self` to a byte slice.
    #[must_use]
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.slot().as_bytes()
    }

    /// Converts `self` to a string slice.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        self.slot().as_str()
    }
}

impl<const N: usize> core::fmt::Debug for StringRef<'_, N> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl<const N: usize> core::fmt::Display for StringRef<'_, N> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const N: usize> PartialEq for StringRef<'_, N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<const N: usize> PartialEq<&str> for StringRef<'_, N> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl<const N: usize> PartialEq<StringRef<'_, N>> for &str {
    #[inline]
    fn eq(&self, other: &StringRef<'_, N>) -> bool {
        *self == other.as_str()
    }
}

impl<const N: usize> core::hash::Hash for StringRef<'_, N> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<const N: usize> Eq for StringRef<'_, N> {}

impl<const N: usize> Clone for StringRef<'_, N> {
    #[inline]
    fn clone(&self) -> Self {
        self.slot().inc_refcount();
        Self {
            slot: self.slot,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<const N: usize> Drop for StringRef<'_, N> {
    #[inline]
    fn drop(&mut self) {
        let slot = self.slot_mut();

        slot.dec_refcount();
        if slot.refcount.get() == 0 {
            slot.release();
        }
    }
}
