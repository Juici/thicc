use core::fmt;

mod convert;

pub use self::convert::{Chars, DecodeWideError};

/// A system wide character, `wchar_t`.
pub type WChar = libc::wchar_t;

mod private {
    pub trait Sealed {}
}

/// A trait representing a UTF wide character.
pub trait Wide:
    private::Sealed
    + Copy
    + Eq
    + Ord
    + fmt::Display
    + fmt::Debug
    + fmt::Binary
    + fmt::LowerHex
    + fmt::UpperHex
    + fmt::Octal
    + convert::Decode
    + wmemchr::Wide
    + 'static
{
    /// The NUL control character.
    const NUL: Self;
}

macro_rules! impl_wide {
    ($($ty:ident)*) => {
        $(
            impl private::Sealed for $ty {}

            impl Wide for $ty {
                const NUL: $ty = 0;
            }
        )*
    };
}
impl_wide!(u16 u32 i16 i32);

assert_impls!(WChar: Wide);

pub(crate) trait SpecLen: Wide {
    unsafe fn wcslen(buf: *const Self) -> usize;
}

pub(crate) trait SpecFind: Wide {
    fn wmemchr(needle: Self, haystack: &[Self]) -> Option<usize>;
}

impl<T: Wide> SpecLen for T {
    default unsafe fn wcslen(buf: *const Self) -> usize {
        let mut end = buf;
        while *end != T::NUL {
            end = end.add(1);
        }
        end.offset_from(buf) as usize
    }
}

impl<T: Wide> SpecFind for T {
    #[inline]
    default fn wmemchr(needle: Self, haystack: &[Self]) -> Option<usize> {
        wmemchr::wmemchr(needle, haystack)
    }
}

impl SpecLen for WChar {
    #[inline]
    unsafe fn wcslen(buf: *const Self) -> usize {
        libc::wcslen(buf)
    }
}

// TODO: Use libc::wmemchr implementation if the target platform provides a
//       good implementation.
// impl SpecFind for WChar {
//     #[inline]
//     fn wmemchr(needle: Self, haystack: &[Self]) -> Option<usize> {
//         let p = unsafe { libc::wmemchr(haystack.as_ptr(), needle, haystack.len()) };
//         if p.is_null() {
//             None
//         } else {
//             Some(unsafe { p.offset_from(haystack.as_ptr()) } as usize)
//         }
//     }
// }
