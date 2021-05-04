#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_raw_ptr_deref)]
#![feature(extern_types)]
#![feature(min_specialization)]

#[macro_use]
mod macros;

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
