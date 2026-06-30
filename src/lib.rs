#![no_std]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(deprecated_in_future)]
#![warn(unused_lifetimes)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![warn(clippy::restriction)]
#![warn(clippy::nursery)]
#![allow(clippy::allow_attributes_without_reason)]
#![allow(clippy::doc_include_without_cfg)]
#![allow(clippy::blanket_clippy_restriction_lints)]
#![allow(clippy::pub_with_shorthand)]
#![allow(clippy::question_mark_used)]
#![allow(clippy::implicit_return)]
#![allow(clippy::pub_use)]
#![allow(clippy::arbitrary_source_item_ordering)]
#![allow(clippy::absolute_paths)]
#![allow(clippy::field_scoped_visibility_modifiers)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_trait_methods)]
#![allow(clippy::redundant_pub_crate)]
#![allow(clippy::single_call_fn)]

mod error;
pub use error::StringPoolError;

mod slot;

mod pool;
pub use pool::StringPool;

mod string_ref;
pub use string_ref::StringRef;

#[cfg(test)]
mod tests {
    use super::{StringPool, StringPoolError};
    use core::cell::UnsafeCell;

    fn is_free<const N: usize, const M: usize>(pool: &StringPool<N, M>, idx: usize) -> bool {
        // SAFETY: there are no other **references** at the moment pointing to the value in the slot
        //         so it's safe to temporarily get one here.
        let slot = unsafe { &*UnsafeCell::raw_get(&pool.slots[idx]) };
        slot.free
    }

    #[test]
    fn test_string_pool() {
        let pool = StringPool::<5, 10>::new();

        let s1 = pool.alloc("foo").unwrap();
        assert_eq!(s1, "foo");
        assert!(!is_free(&pool, 0));

        let s2 = pool.alloc("bar").unwrap();
        assert_eq!(s2, "bar");
        assert!(!is_free(&pool, 1));

        drop(s2);
        assert!(is_free(&pool, 1));

        drop(s1);
        assert!(is_free(&pool, 0));
    }

    #[test]
    fn test_string_pool_reuse() {
        let pool = StringPool::<5, 10>::new();

        for _ in 0..2 {
            let _s1 = pool.alloc("one").unwrap();
            let _s2 = pool.alloc("two").unwrap();
            let _s3 = pool.alloc("three").unwrap();
            let _s4 = pool.alloc("four").unwrap();
            let _s5 = pool.alloc("five").unwrap();

            for idx in 0..5 {
                assert!(
                    !is_free(&pool, idx),
                    "expected slot at {idx} to be occupied"
                );
            }
        }
    }

    #[test]
    fn test_pool_overflow() {
        let pool = StringPool::<3, 10>::new();

        let _s1 = pool.alloc("one").unwrap();
        let _s2 = pool.alloc("two").unwrap();
        let s3 = pool.alloc("three").unwrap();
        assert!(pool.alloc("four").is_err());

        drop(s3);
        let _s4 = pool.alloc("four").unwrap();
    }

    #[test]
    fn test_slot_overflow() {
        let pool = StringPool::<1, 5>::new();

        assert!(pool.alloc("123456").is_err());
        let _s = pool.alloc("12345").unwrap();

        assert!(!is_free(&pool, 0));
        {
            // SAFETY: `s` holds a pointer to a slot, not a reference,
            //         so it safe to temporarily create one here.
            let slot = unsafe { &*UnsafeCell::raw_get(&pool.slots[0]) };
            assert_eq!(slot.str, [b'1', b'2', b'3', b'4', b'5']);
        }
    }

    #[test]
    fn test_zero_size_slots() {
        let pool = StringPool::<1, 0>::new();

        let err = pool.alloc("f").unwrap_err();
        assert_eq!(
            err,
            StringPoolError::StringIsTooLong {
                max_length: 0,
                actual_length: 1
            }
        );

        let err = pool.alloc("").unwrap_err();
        assert_eq!(
            err,
            StringPoolError::StringIsTooLong {
                max_length: 0,
                actual_length: 0
            }
        );
    }

    #[test]
    fn test_drop_clone_while_str_borrow_is_alive() {
        let pool = StringPool::<1, 10>::new();

        let s1 = pool.alloc("foo").unwrap();
        let s2 = s1.clone();

        let borrowed = s2.as_str();
        drop(s1);

        assert_eq!(borrowed, "foo");
    }
}
