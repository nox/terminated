/*!
This crate provides types for representing NUL-terminated UTF8 strings.

The `NulTerminatedStr` type is useful when interacting with C APIs that
require/guarantee UTF8 encoding. Rust has great support for dealing with UTF8,
but C strings require a NUL terminator which Rust's `str` and `String` don't have.

```
# #![feature(use_extern_macros)]
# #[macro_use] extern crate terminated;
# fn main() {
let s = ntstr!("Hello, World!");

// You can use Rust's normal string operations
assert_eq!(s.find("World"), Some(7));

// And pass it to C since it's NUL-terminated
let ptr = s.as_ptr();
# }
```

# CStr vs NulTerminatedStr

The standard library does provide the `CStr` type that is NUL-terminated,
but it does not use any specific encoding. It's therefore insufficient
if your input needs to be both NUL-terminated and UTF8 encoded.
*/

#![cfg_attr(terminated_unstable, feature(use_extern_macros))]
#![no_std]

#[cfg(terminated_unstable)]
#[doc(hidden)]
pub extern crate terminated_macros;

use core::fmt;
use core::mem;
use core::ops::Deref;

/// An error indicating that a string is not correctly NUL-terminated.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NulError {
    /// The string contained an interior NUL at this position.
    InteriorNul(usize),
    /// The string did not have a NUL as its last character.
    NotNulTerminated,
}

impl fmt::Display for NulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NulError::InteriorNul(pos) =>
                write!(f, "data provided contains an interior nul at byte pos {}", pos),
            NulError::NotNulTerminated =>
                f.write_str("data provided is not nul terminated"),
        }
    }
}

/**
A valid UTF8 string terminated by NUL, the null character.

`NulTerminatedStr` dereferences to a `str` slice excluding the NUL terminator,
meaning all of `str`'s methods are available:

```
# #![feature(use_extern_macros)]
# #[macro_use] extern crate terminated;
# fn main() {
let s = ntstr!("Hello, World!");
assert_eq!(s.len(), 13);
assert!(s.starts_with("Hello"));
assert!(s.ends_with("World!"));
assert_eq!(s.find("World"), Some(7));
# }
```
*/
pub struct NulTerminatedStr(str);

impl NulTerminatedStr {
    /**
    Creates a `NulTerminatedStr` from a given string that is NUL-terminated.

    If the given string is not correctly NUL-terminated, a `NulError` is returned.

    # Example
    ```
    # use terminated::NulTerminatedStr;
    let mut s = "Hello, World!".to_string();
    s.push('\0');

    let nts = NulTerminatedStr::from_str_with_nul(&s);
    assert!(nts.is_ok());
    ```
    */
    pub fn from_str_with_nul(s: &str) -> Result<&NulTerminatedStr, NulError> {
        let nul_pos = s.bytes().position(|b| b == 0);
        nul_pos.ok_or(NulError::NotNulTerminated).and_then(|i| {
            // The first (and only) nul must be at the last index
            if i == s.len() - 1 {
                Ok(unsafe { mem::transmute(s) })
            } else {
                Err(NulError::InteriorNul(i))
            }
        })
    }

    pub unsafe fn from_str_with_nul_unchecked(s: &str) -> &NulTerminatedStr {
        &*(s as *const str as *const NulTerminatedStr)
    }

    /// Returns the content of self including the NUL terminator.
    pub fn as_str_with_nul(&self) -> &str {
        &self.0
    }
}

impl Deref for NulTerminatedStr {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0[..self.0.len() - 1]
    }
}

impl AsRef<str> for NulTerminatedStr {
    fn as_ref(&self) -> &str {
        &**self
    }
}

impl fmt::Debug for NulTerminatedStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str_with_nul(), f)
    }
}

impl fmt::Display for NulTerminatedStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

/**
Creates a static `NulTerminatedStr` from a string literal.

# Example
```
# #![feature(use_extern_macros)]
# #[macro_use] extern crate terminated;
# fn main() {
let s = ntstr!("Hello, World!");
assert_eq!(s.as_str_with_nul(), "Hello, World!\0");
# }
```
*/
#[macro_export]
macro_rules! ntstr { ($e:expr) => (ntstr_impl!($e)) }

#[cfg(not(terminated_unstable))]
#[doc(hidden)]
#[macro_export]
macro_rules! ntstr_impl {
    ($e:expr) => (
        match $crate::NulTerminatedStr::from_str_with_nul(concat!($e, "\0")) {
            Ok(s) => s,
            Err(e) => panic!("{}", e),
        }
    )
}

#[cfg(terminated_unstable)]
#[doc(hidden)]
#[macro_export]
macro_rules! ntstr_impl {
    ($e:expr) => (
        {
            #[allow(unsafe_code)]
            unsafe {
                $crate::NulTerminatedStr::from_str_with_nul_unchecked(
                    $crate::terminated_macros::ntstr!($e),
                )
            }
        }
    )
}

#[cfg(test)]
mod tests {
    use super::{NulTerminatedStr, NulError};

    #[test]
    fn test() {
        let s = ntstr!("foo");
        assert_eq!(&**s, "foo");
        assert_eq!(s.len(), 3);
        assert_eq!(&s[0..2], "fo");
        assert_eq!(s.as_str_with_nul(), "foo\0");
    }

    #[test]
    fn test_err() {
        assert_eq!(NulTerminatedStr::from_str_with_nul("foo").unwrap_err(),
            NulError::NotNulTerminated);
        assert_eq!(NulTerminatedStr::from_str_with_nul("fo\0o").unwrap_err(),
            NulError::InteriorNul(2));
        assert_eq!(NulTerminatedStr::from_str_with_nul("fo\0o\0").unwrap_err(),
            NulError::InteriorNul(2));
    }
}
