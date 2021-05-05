use core::fmt;
use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::mem;

use crate::char::Wide;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodeWideError<T> {
    code: T,
}

pub trait Decode: Copy + Eq + Ord + 'static {
    fn next(iter: &mut Chars<'_, Self>) -> Option<Result<char, DecodeWideError<Self>>>;
    fn size_hint(wcslen: usize) -> (usize, Option<usize>);
}

pub struct Chars<'a, T> {
    ptr: *const T,
    _marker: PhantomData<&'a T>,
}

macro_rules! wcslen {
    ($self:ident) => {{
        // Sometimes used within an unsafe block.
        #![allow(unused_unsafe)]

        // SAFETY: Safe references can only exist if they point to memory that
        //         has a NUL-terminator.
        unsafe { $crate::char::SpecWide::wcslen($self.ptr) }
    }};
}

macro_rules! impl_decode_utf16 {
    ($($ty:ident)*) => {
        $(
            static_assert!(mem::size_of::<$ty>() == mem::size_of::<u16>());
            static_assert!(mem::align_of::<$ty>() == mem::align_of::<u16>());

            impl Decode for $ty {
                #[inline]
                fn next(iter: &mut Chars<'_, Self>) -> Option<Result<char, DecodeWideError<$ty>>> {
                    // SAFETY: Safe references to `Chars` can only exist if they point to
                    //         memory that has a NUL-terminator.
                    let u = unsafe { *(iter.ptr as *const u16) };
                    // Check if at NUL-terminator.
                    if u == u16::NUL {
                        return None;
                    }
                    // SAFETY: Not yet at NUL-terminator.
                    iter.ptr = unsafe { iter.ptr.add(1) };

                    if u < 0xD800 || 0xDFFF < u {
                        // SAFETY: Not a surrogate.
                        Some(Ok(unsafe { char::from_u32_unchecked(u as u32) }))
                    } else if u >= 0xDC00 {
                        // A trailing surrogate.
                        Some(Err(DecodeWideError { code: unsafe { mem::transmute(u) } }))
                    } else {
                        // SAFETY: Safe references to `Chars` can only exist if they point to
                        //         memory that has a NUL-terminator.
                        let u2 = unsafe { *(iter.ptr as *const u16) };
                        // Check if missing trailing surrogate.
                        if u2 == u16::NUL || u2 < 0xDC00 || u2 > 0xDFFF {
                            return Some(Err(DecodeWideError { code: unsafe { mem::transmute(u) } }));
                        }
                        // SAFETY: Not yet at NUL-terminator.
                        iter.ptr = unsafe { iter.ptr.add(1) };

                        // All ok, so lets decode it.
                        let c = (((u - 0xD800) as u32) << 10 | (u2 - 0xDC00) as u32) + 0x1_0000;
                        // SAFETY: We checked that it's a legal unicode value.
                        Some(Ok(unsafe { char::from_u32_unchecked(c) }))
                    }
                }

                #[inline]
                fn size_hint(wcslen: usize) -> (usize, Option<usize>) {
                    // The iterator could be entirely valid surrogates (2 elements per char),
                    // or entirely non-surrogates (1 element per char).
                    (wcslen / 2, Some(wcslen))
                }
            }
        )*
    };
}
impl_decode_utf16!(u16 i16);

macro_rules! impl_decode_utf32 {
    ($($ty:ident)*) => {
        $(
            static_assert!(mem::size_of::<$ty>() == mem::size_of::<u32>());
            static_assert!(mem::align_of::<$ty>() == mem::align_of::<u32>());

            impl Decode for $ty {
                #[inline]
                fn next(iter: &mut Chars<'_, Self>) -> Option<Result<char, DecodeWideError<$ty>>> {
                    // SAFETY: Safe references to `Chars` can only exist if they point to
                    //         memory that has a NUL-terminator.
                    let u = unsafe { *(iter.ptr as *const u32) };
                    // Check if at NUL-terminator.
                    if u == u32::NUL {
                        return None;
                    }
                    // SAFETY: Not yet at NUL-terminator.
                    iter.ptr = unsafe { iter.ptr.add(1) };

                    match char::from_u32(u) {
                        Some(c) => Some(Ok(c)),
                        None => Some(Err(DecodeWideError { code: unsafe { mem::transmute(u) } }))
                    }
                }

                #[inline]
                fn size_hint(wcslen: usize) -> (usize, Option<usize>) {
                    (wcslen, Some(wcslen))
                }
            }

            impl ExactSizeIterator for Chars<'_, $ty> {
                #[inline(always)]
                fn len(&self) -> usize {
                    wcslen!(self)
                }
            }
        )*
    };
}
impl_decode_utf32!(u32 i32);

impl<'a, T: Wide> Chars<'a, T> {
    #[inline]
    pub(crate) fn new(ptr: *const T) -> Chars<'a, T> {
        Chars {
            ptr,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: Wide> Iterator for Chars<'a, T> {
    type Item = Result<char, DecodeWideError<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        T::next(self)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        T::size_hint(wcslen!(self))
    }
}

impl<T: Wide> FusedIterator for Chars<'_, T> {}

impl<T: Wide> DecodeWideError<T> {
    /// Returns the wide character that caused this error.
    #[inline]
    pub fn code(&self) -> T {
        self.code
    }
}

impl<T: Wide> fmt::Display for DecodeWideError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to decode wide character: {:x}", self.code)
    }
}

#[cfg(feature = "std")]
impl<T: Wide> std::error::Error for DecodeWideError<T> {}
