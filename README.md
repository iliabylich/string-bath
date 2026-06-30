# Fixed-size single-threaded `no_std` zero-dependency string pool implementation

Usage example:

```rust
use string_bath::{StringPool, StringPoolError, StringRef};

// 2 slots, each may store at most 5 bytes
let pool = StringPool::<2, 5>::new();

let foo: StringRef<5> = pool.alloc("foo").unwrap();
assert_eq!(foo, "foo");

let bar: StringRef<5> = pool.alloc("bar").unwrap();
assert_eq!(bar, "bar");

// 2 strings have been constructed, the next `alloc()` call is guaranteed to fail
assert_eq!(pool.alloc("baz").unwrap_err(), StringPoolError::NoSpaceInPool);

// but if we drop one of the strings we can re-use the slot in the pool
drop(foo);
let baz: StringRef<5> = pool.alloc("baz").unwrap();
assert_eq!(baz, "baz");
```

## Compatibility with C

Each slot reserves 1 byte for the NULL terminator, which makes each `StringRef` object conversible to a C-style `char*`:

```rust
use string_bath::{StringPool, StringPoolError, StringRef};

// 1 slot, max string length = 5
let pool = StringPool::<1, 5>::new();
assert_eq!(
    pool.alloc("123456").unwrap_err(),
    StringPoolError::StringIsTooLong { max_length: 5, actual_length: 6 }
);

let s: StringRef<5> = pool.alloc("12345").unwrap();
let ptr = unsafe { core::mem::transmute::<StringRef<5>, *const i8>(s) };
let c_str = unsafe { core::ffi::CStr::from_ptr(ptr) };
assert_eq!(c_str.to_str().unwrap(), "12345");
```
