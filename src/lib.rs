#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_fn)]
#![feature(const_raw_ptr_deref)]
#![feature(extern_types)]
#![feature(min_specialization)]

cfg_if::cfg_if! {
    if #[cfg(feature = "macros")] {
        #[doc(hidden)]
        pub use wchar as _wchar;

        #[macro_use]
        mod macros;
    }
}

#[macro_use]
mod internal_macros;

mod char;
// mod wstr;
mod wcstr;

pub use crate::char::{Chars, DecodeWideError, WChar, Wide};
pub use crate::wcstr::{FromSliceWithNulError, WCStr};

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        mod alloc;
        // mod wstring;
        // mod wcstring;
    }
}
