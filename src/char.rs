use core::fmt;
use core::mem;
use core::option::Option::Some;
use core::slice;

/// A system wide character, `wchar_t`.
pub type WChar = libc::wchar_t;

/// A trait representing a UTF wide character.
pub trait Wide: Copy + Eq + Ord + 'static {
    /// The NUL control character.
    const NUL: Self;

    #[doc(hidden)]
    fn decode_next(iter: &mut slice::Iter<'_, Self>) -> Option<Result<char, DecodeWideError>>;
    #[doc(hidden)]
    fn size_hint(iter: &slice::Iter<'_, Self>) -> (usize, Option<usize>);
}

macro_rules! impl_utf16 {
    ($($ty:ident)*) => {
        $(
            static_assert!(mem::size_of::<$ty>() == mem::size_of::<u16>());

            impl Wide for $ty {
                const NUL: $ty = 0;

                fn decode_next(
                    iter: &mut slice::Iter<'_, $ty>,
                ) -> Option<Result<char, DecodeWideError>> {
                    let u = *iter.next()? as u16;

                    if u < 0xD800 || 0xDFFF < u {
                        // SAFETY: Not a surrogate.
                        Some(Ok(unsafe { char::from_u32_unchecked(u as u32) }))
                    } else if u >= 0xDC00 {
                        // A trailing surrogate.
                        Some(Err(DecodeWideError(())))
                    } else {
                        let u2 = match iter.as_slice().first() {
                            // Not a trailing surrogate so we're not a valid surrogate pair.
                            Some(&u2) if (u2 as u16) < 0xDC00 || (u2 as u16) > 0xDFFF => {
                                return Some(Err(DecodeWideError(())));
                            }
                            Some(_) => *iter.next()? as u16,
                            // Missing trailing surrogate.
                            None => return Some(Err(DecodeWideError(()))),
                        };

                        // All ok, so lets decode it.
                        let c = (((u - 0xD800) as u32) << 10 | (u2 - 0xDC00) as u32) + 0x1_0000;
                        // SAFETY: We checked that it's a legal unicode value.
                        Some(Ok(unsafe { char::from_u32_unchecked(c) }))
                    }
                }

                #[inline]
                fn size_hint(iter: &slice::Iter<'_, $ty>) -> (usize, Option<usize>) {
                    let len = iter.len();
                    // The iterator could be entirely valid surrogates (2 elements per char),
                    // or entirely non-surrogates (1 element per char).
                    (len / 2, Some(len))
                }
            }
        )*
    };
}
impl_utf16!(u16 i16);

macro_rules! impl_utf32 {
    ($($ty:ident)*) => {
        $(
            static_assert!(mem::size_of::<$ty>() == mem::size_of::<u32>());

            impl Wide for $ty {
                const NUL: $ty = 0;

                fn decode_next(
                    iter: &mut slice::Iter<'_, $ty>,
                ) -> Option<Result<char, DecodeWideError>> {
                    let u = *iter.next()? as u32;
                    match char::from_u32(u) {
                        Some(c) => Some(Ok(c)),
                        None => Some(Err(DecodeWideError(()))),
                    }
                }

                #[inline]
                fn size_hint(iter: &slice::Iter<'_, $ty>) -> (usize, Option<usize>) {
                    let len = iter.len();
                    (len, Some(len))
                }
            }
        )*
    };
}
impl_utf32!(u32 i32);

assert_impls!(WChar: Wide);

pub trait SpecWide: Wide {
    unsafe fn wcslen(buf: *const Self) -> usize;
    fn wmemchr(needle: Self, haystack: &[Self]) -> Option<usize>;
}

impl<T: Wide> SpecWide for T {
    default unsafe fn wcslen(buf: *const Self) -> usize {
        let mut len = 0;
        while *buf.add(len) != T::NUL {
            len += 1;
        }
        len
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

#[derive(Clone, Debug)]
pub struct DecodeWideError(());

impl fmt::Display for DecodeWideError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("failed to decode wide character", f)
    }
}

// TODO: Manual pointer iterator, until NUL-terminator.
//       Preventing the need to find the length before iterating.
#[derive(Clone)]
pub struct Chars<'a, T: Wide> {
    pub(crate) iter: slice::Iter<'a, T>,
}

impl<'a, T: Wide> Iterator for Chars<'a, T> {
    type Item = Result<char, DecodeWideError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        T::decode_next(&mut self.iter)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        T::size_hint(&self.iter)
    }
}

impl ExactSizeIterator for Chars<'_, u32> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
impl ExactSizeIterator for Chars<'_, i32> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
