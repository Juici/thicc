use core::mem;
use core::slice;

use crate::char::{Chars, SpecWide, WChar};

extern "C" {
    // HACK: Extern type to prevent `WCStr` from being sized.
    type WCStrExtern;
}

/// A C-style wide character string.
#[repr(transparent)]
pub struct WCStr(WCStrExtern);

assert_impls!(WCStr: !Sized);
static_assert!(mem::size_of::<&WCStr>() == mem::size_of::<*const WChar>());
static_assert!(mem::align_of::<&WCStr>() == mem::align_of::<*const WChar>());

impl WCStr {
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
    #[inline]
    pub const unsafe fn from_ptr<'a>(ptr: *const WChar) -> &'a WCStr {
        &*(ptr as *const WCStr)
    }

    /// Creates a `WCStr` from a byte slice.
    ///
    /// This function will cast the provided `slice` to a `CStr`
    /// wrapper after ensuring that the byte slice is NUL-terminated
    /// and does not contain any interior NUL bytes.
    pub fn from_slice_with_nul(slice: &[WChar]) -> Result<&WCStr, FromSliceWithNulError> {
        let nul_pos = SpecWide::wmemchr(0, slice);
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
    #[inline]
    pub const unsafe fn from_slice_with_nul_unchecked(slice: &[WChar]) -> &WCStr {
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
    #[inline]
    pub const fn as_ptr(&self) -> *const WChar {
        self as *const WCStr as *const WChar
    }

    /// Converts a `WCStr` into a slice of wide characters.
    ///
    /// The returned slice will **not** contain the trailing NUL-terminator.
    ///
    /// > **Note**: This operation is not zero-cost, requiring iteration through
    /// > all bytes of the string to calculate the length.
    #[inline]
    pub fn to_slice(&self) -> &[WChar] {
        // SAFETY: Safe references to `WCStr` can only exist if they point to
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
    #[inline]
    pub fn to_slice_with_nul(&self) -> &[WChar] {
        // SAFETY: Safe references to `WCStr` can only exist if they point to
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
    #[inline]
    pub fn len(&self) -> usize {
        // SAFETY: Safe references to `WCStr` can only exist if they point to
        //         memory that has a NUL-terminator.
        unsafe { SpecWide::wcslen(self.as_ptr()) }
    }

    /// Returns an iterator over the [`char`]s of a wide string.
    pub fn chars(&self) -> Chars<'_, WChar> {
        // SAFETY: Safe references to `WCStr` can only exist if they point to
        //         memory that has a NUL-terminator.
        Chars {
            iter: self.to_slice().iter(),
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
