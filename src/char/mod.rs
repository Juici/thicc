use core::fmt;

mod convert;

pub use self::convert::{Chars, DecodeWideError};

/// A system wide character, `wchar_t`.
pub type WChar = libc::wchar_t;

/// A trait representing a UTF wide character.
pub trait Wide:
    Copy
    + Eq
    + Ord
    + fmt::Display
    + fmt::Debug
    + fmt::Binary
    + fmt::LowerHex
    + fmt::UpperHex
    + fmt::Octal
    + convert::Decode
    + 'static
{
    /// The NUL control character.
    const NUL: Self;
}

macro_rules! impl_wide {
    ($($ty:ident)*) => {
        $(
            impl Wide for $ty {
                const NUL: $ty = 0;
            }
        )*
    };
}
impl_wide!(u16 u32 i16 i32);

assert_impls!(WChar: Wide);

pub trait SpecWide: Wide {
    unsafe fn wcslen(buf: *const Self) -> usize;
    fn wmemchr(needle: Self, haystack: &[Self]) -> Option<usize>;
}

impl<T: Wide> SpecWide for T {
    default unsafe fn wcslen(buf: *const Self) -> usize {
        let mut end = buf;
        while *end != T::NUL {
            end = end.add(1);
        }
        end.offset_from(buf) as usize
    }

    default fn wmemchr(needle: Self, haystack: &[Self]) -> Option<usize> {
        let mut pos = 0;
        for &c in haystack {
            if c == needle {
                return Some(pos);
            }
            pos += 1;
        }
        None
    }
}

impl SpecWide for WChar {
    #[inline]
    unsafe fn wcslen(buf: *const Self) -> usize {
        libc::wcslen(buf)
    }

    #[inline]
    fn wmemchr(needle: Self, haystack: &[Self]) -> Option<usize> {
        let p = unsafe { libc::wmemchr(haystack.as_ptr(), needle, haystack.len()) };
        if p.is_null() {
            None
        } else {
            Some(unsafe { p.offset_from(haystack.as_ptr()) } as usize)
        }
    }
}
