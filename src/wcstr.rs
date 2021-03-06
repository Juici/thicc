use core::marker::PhantomData;
use core::mem;
use core::slice;

use crate::char::{Chars, WChar, Wide};

extern "C" {
    // HACK: Extern type to prevent `WCStr` from being sized.
    type WCStrExtern;
}

/// A C-style wide character string.
#[repr(transparent)]
pub struct WCStr<T: Wide = WChar>(PhantomData<T>, WCStrExtern);

assert_impls!(WCStr: !Sized);
static_assert!(mem::size_of::<&WCStr>() == mem::size_of::<*const WChar>());
static_assert!(mem::size_of::<&WCStr>() == mem::size_of::<*mut WChar>());
static_assert!(mem::align_of::<&WCStr>() == mem::align_of::<*const WChar>());
static_assert!(mem::align_of::<&WCStr>() == mem::align_of::<*mut WChar>());

macro_rules! assert_repr {
    ($($ty:ident)*) => {
        $(
            assert_impls!(WCStr<$ty>: !Sized);
            static_assert!(mem::size_of::<&WCStr<$ty>>() == mem::size_of::<*const $ty>());
            static_assert!(mem::size_of::<&WCStr<$ty>>() == mem::size_of::<*mut $ty>());
            static_assert!(mem::align_of::<&WCStr<$ty>>() == mem::align_of::<*const $ty>());
            static_assert!(mem::align_of::<&WCStr<$ty>>() == mem::align_of::<*mut $ty>());
        )*
    };
}
assert_repr!(u16 u32 i16 i32);

impl<T: Wide> WCStr<T> {
    /// Creates a `WCStr` from a raw pointer to a C-style wide string.
    ///
    /// This
    ///
    /// # Safety
    ///
    /// - `ptr` must be non-null.
    ///
    /// - The memory referenced by `ptr` must be valid for the returned
    ///   lifetime.
    ///
    /// - The memory referenced by `ptr` must have a NUL-terminator character
    ///   within the allocation.
    ///
    /// - The memory referenced by `ptr` must not be modified before the
    ///   returned `WCStr` is dropped.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use thicc::{WChar, WCStr};
    ///
    /// # extern "C" fn my_string() -> *const WChar {
    /// #    static STRING: &[WChar] = wchar::wchz!("hello world!");
    /// #    STRING.as_ptr()
    /// # }
    /// # const _: fn() = || {
    /// extern "C" {
    ///     fn my_string() -> *const WChar;
    /// }
    /// # };
    ///
    /// unsafe {
    ///     let s = WCStr::from_ptr(my_string());
    ///
    ///     println!("string returned: {}", s.to_string_lossy());
    /// }
    /// ```
    #[inline]
    pub const unsafe fn from_ptr<'a>(ptr: *const T) -> &'a WCStr<T> {
        &*(ptr as *const WCStr<T>)
    }

    /// Creates a `WCStr` from a byte slice.
    ///
    /// This function will cast the provided `slice` to a `CStr`
    /// wrapper after ensuring that the byte slice is NUL-terminated
    /// and does not contain any interior NUL bytes.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use thicc::WCStr;
    ///
    /// // ????<nul>
    /// let v: &[u16] = &[0xD83D, 0xDC96, 0x0000];
    /// let s = WCStr::from_slice_with_nul(v).unwrap();
    ///
    /// assert_eq!(s.chars().next(), Some(Ok('????')));
    ///
    /// // ????<nul>
    /// let v: &[u32] = &[0x0001_F980, 0x0000_0000];
    /// let s = WCStr::from_slice_with_nul(v).unwrap();
    ///
    /// assert_eq!(s.chars().next(), Some(Ok('????')));
    /// ```
    ///
    /// Creating a `WCStr` without a NUL-terminator is an error:
    ///
    /// ```
    /// use thicc::WCStr;
    ///
    /// // ????
    /// let v: &[u32] = &[0x0001_F496];
    /// assert!(WCStr::from_slice_with_nul(v).is_err());
    /// ```
    ///
    /// Creating a `WCStr` with an interior NUL character is an error:
    ///
    /// ```
    /// use thicc::WCStr;
    ///
    /// // ????<nul>????<nul>
    /// let v: &[u16] = &[0xD83E, 0xDD80, 0x0000, 0xD83E, 0xDD80, 0x0000];
    /// assert!(WCStr::from_slice_with_nul(v).is_err());
    /// ```
    pub fn from_slice_with_nul(slice: &[T]) -> Result<&WCStr<T>, FromSliceWithNulError> {
        use crate::char::SpecFind;

        let nul_pos = SpecFind::wmemchr(T::NUL, slice);
        if let Some(nul_pos) = nul_pos {
            if nul_pos + 1 != slice.len() {
                return Err(FromSliceWithNulError::interior_nul(nul_pos));
            }
            Ok(unsafe { WCStr::from_slice_with_nul_unchecked(slice) })
        } else {
            Err(FromSliceWithNulError::not_nul_terminated())
        }
    }

    /// Creates a `WCStr` from a slice of wide characters with a NUL-terminator.
    ///
    /// No checks are performed that `slice` is a valid `WCStr`.
    ///
    /// # Safety
    ///
    /// `slice` must be NUL-terminated and cannot contain any interior NUL
    /// characters.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use thicc::WCStr;
    /// use wchar::wchz;
    ///
    /// unsafe {
    ///     let s = WCStr::from_slice_with_nul_unchecked(wchz!("????????????"));
    ///
    ///     let mut iter = s.chars();
    ///
    ///     assert_eq!(iter.next(), Some(Ok('????')));
    ///     assert_eq!(iter.next(), Some(Ok('????')));
    ///     assert_eq!(iter.next(), Some(Ok('????')));
    ///     assert_eq!(iter.next(), None);
    /// }
    /// ```
    #[inline]
    pub const unsafe fn from_slice_with_nul_unchecked(slice: &[T]) -> &WCStr<T> {
        WCStr::from_ptr(slice.as_ptr())
    }

    /// Returns the inner pointer to this C-style wide string.
    ///
    /// The returned pointer will be valid for as long as `self` is, and points
    /// to a contiguous region of memory with a NUL-terminator to represent the
    /// end of the string.
    ///
    /// **WARNING**
    ///
    /// The returned pointer is read-only; writing to it (including passing it
    /// to C code that writes to it) causes undefined behavior.
    ///
    /// It is your responsibility to make sure that the underlying memory is not
    /// freed too early.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use thicc::{WChar, WCStr};
    /// use wchar::wchz;
    ///
    /// # extern "C" fn my_string(_s: *const WChar) {}
    /// # const _: fn() = || {
    /// extern "C" {
    ///     fn my_string(s: *const WChar);
    /// }
    /// # };
    ///
    /// unsafe {
    ///     let s = WCStr::from_slice_with_nul(wchz!("hello world!")).unwrap();
    ///
    ///     my_string(s.as_ptr());
    /// }
    /// ```
    #[inline]
    pub const fn as_ptr(&self) -> *const T {
        self as *const WCStr<T> as *const T
    }

    /// Converts a `WCStr` into a slice of wide characters.
    ///
    /// The returned slice will **not** contain the trailing NUL-terminator.
    ///
    /// > **Note**: This operation is not zero-cost, requiring iteration through
    /// > all bytes of the string to calculate the length.
    ///
    /// Basic usage:
    ///
    /// ```
    /// use thicc::WCStr;
    ///
    /// // ????<nul>
    /// let v: &[u16] = &[0xD83D, 0xDC96, 0x0000];
    /// let s = WCStr::from_slice_with_nul(v).unwrap();
    ///
    /// assert_eq!(&v[..2], s.to_slice());
    ///
    /// // ????<nul>
    /// let v: &[u32] = &[0x0001_F980, 0x0000_0000];
    /// let s = WCStr::from_slice_with_nul(v).unwrap();
    ///
    /// assert_eq!(&v[..1], s.to_slice());
    /// ```
    #[inline]
    pub fn to_slice(&self) -> &[T] {
        // SAFETY: Safe references to `WCStr<T>` can only exist if they point to
        //         memory that has a NUL-terminator.
        unsafe {
            let len = self.len();
            slice::from_raw_parts(self.as_ptr(), len)
        }
    }

    /// Converts a `WCStr` into a slice of wide characters containing the
    /// trailing NUL-terminator.
    ///
    /// This function is the equivalent of [`WCStr::to_bytes`] except that it
    /// will retain the trailing NUL-terminator instead of chopping it off.
    ///
    /// > **Note**: This operation is not zero-cost, requiring iteration through
    /// > all bytes of the string to calculate the length.
    ///
    /// Basic usage:
    ///
    /// ```
    /// use thicc::WCStr;
    ///
    /// // ????<nul>
    /// let v: &[u16] = &[0xD83E, 0xDD80, 0x0000];
    /// let s = WCStr::from_slice_with_nul(v).unwrap();
    ///
    /// assert_eq!(v, s.to_slice_with_nul());
    ///
    /// // ????<nul>
    /// let v: &[u32] = &[0x0001_F496, 0x0000_0000];
    /// let s = WCStr::from_slice_with_nul(v).unwrap();
    ///
    /// assert_eq!(v, s.to_slice_with_nul());
    /// ```
    #[inline]
    pub fn to_slice_with_nul(&self) -> &[T] {
        // SAFETY: Safe references to `WCStr<T>` can only exist if they point to
        //         memory that has a NUL-terminator.
        unsafe {
            let len = self.len();
            slice::from_raw_parts(self.as_ptr(), len + 1)
        }
    }

    /// Returns the length of a wide string.
    ///
    /// The length is the number of non-NUL wide characters that precede the
    /// NUL-terminator.
    ///
    /// > **Note**: This operation is not zero-cost, requiring iteration through
    /// > all bytes of the string to calculate the length.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use thicc::WCStr;
    ///
    /// // ????music
    /// let v: &[u16] = &[
    ///     0xD834, 0xDD1E, 0x006D, 0x0075, 0x0073, 0x0069, 0x0063, 0x0000,
    /// ];
    /// let s = WCStr::from_slice_with_nul(v).unwrap();
    ///
    /// assert_eq!(s.len(), 7);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        use crate::char::SpecLen;

        // SAFETY: Safe references to `WCStr<T>` can only exist if they point to
        //         memory that has a NUL-terminator.
        unsafe { SpecLen::wcslen(self.as_ptr()) }
    }

    /// Returns an iterator over the [`char`]s of a wide string.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use thicc::WCStr;
    ///
    /// // ????mus<invalid>ic<invalid><nul>
    /// let v: &[u16] = &[
    ///     0xD834, 0xDD1E, 0x006D, 0x0075, 0x0073, 0xDD1E, 0x0069, 0x0063,
    ///     0xD834, 0x0000,
    /// ];
    /// let s = WCStr::from_slice_with_nul(v).unwrap();
    ///
    /// let mut iter = s.chars().map(|r| r.map_err(|e| e.code()));
    ///
    /// assert_eq!(iter.next(), Some(Ok('????')));
    /// assert_eq!(iter.next(), Some(Ok('m')));
    /// assert_eq!(iter.next(), Some(Ok('u')));
    /// assert_eq!(iter.next(), Some(Ok('s')));
    /// assert_eq!(iter.next(), Some(Err(0xDD1E)));
    /// assert_eq!(iter.next(), Some(Ok('i')));
    /// assert_eq!(iter.next(), Some(Ok('c')));
    /// assert_eq!(iter.next(), Some(Err(0xD834)));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn chars(&self) -> Chars<'_, T> {
        // SAFETY: Safe references to `WCStr<T>` can only exist if they point to
        //         memory that has a NUL-terminator.
        Chars::new(self.as_ptr())
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        use crate::alloc::String;

        impl<T: Wide> WCStr<T> {
            /// Decodes a wide character string into a [`String`], replacing
            /// invalid data with [the replacement character (`U+FFFD`)][U+FFFD].
            ///
            /// [U+FFFD]: char::REPLACEMENT_CHARACTER
            ///
            /// # Examples
            ///
            /// Basic usage:
            ///
            /// ```
            /// use thicc::WCStr;
            ///
            /// // ????mus<invalid>ic<invalid>
            /// let v: &[u16] = &[
            ///     0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0xDD1E, 0x0069, 0x0063, 0xD834, 0x0000,
            /// ];
            /// let s = WCStr::from_slice_with_nul(v).unwrap();
            ///
            /// assert_eq!("????mus\u{FFFD}ic\u{FFFD}", s.to_string_lossy());
            /// ```
            #[inline]
            pub fn to_string_lossy(&self) -> String {
                self.chars().map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER)).collect()
            }
        }
    }
}

/// An error indicating that a NUL byte was not in the expected position.
///
/// The slice used to create a [`WCStr`] must have one and only one NUL
/// character, positioned at the end.
///
/// This error is created by the [`WCStr::from_slice_with_nul`] method.
/// See its documentation for more.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FromSliceWithNulError {
    kind: FromSliceWithNulErrorKind,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum FromSliceWithNulErrorKind {
    InteriorNul(usize),
    NotNulTerminated,
}

impl FromSliceWithNulError {
    const fn interior_nul(pos: usize) -> FromSliceWithNulError {
        FromSliceWithNulError {
            kind: FromSliceWithNulErrorKind::InteriorNul(pos),
        }
    }

    const fn not_nul_terminated() -> FromSliceWithNulError {
        FromSliceWithNulError {
            kind: FromSliceWithNulErrorKind::NotNulTerminated,
        }
    }
}
