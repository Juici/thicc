/// Creates a `WCStr` from a string literal at compile time.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use thicc::{WCStr, wcstr};
///
/// const RUST: &WCStr = wcstr!("Rust");
///
/// assert_eq!("Rust", RUST.to_string_lossy());
/// ```
///
/// UTF-16 usage:
///
/// ```
/// use thicc::{WCStr, wcstr};
///
/// const RUST: &WCStr<u16> = wcstr!(u16, "Rust");
/// const ALSO_RUST: &WCStr<i16> = wcstr!(i16, "Rust");
///
/// assert_eq!("Rust", RUST.to_string_lossy());
/// assert_eq!("Rust", ALSO_RUST.to_string_lossy());
/// ```
///
/// UTF-32 usage:
///
/// ```
/// use thicc::{WCStr, wcstr};
///
/// const RUST: &WCStr<u32> = wcstr!(u32, "Rust");
/// const ALSO_RUST: &WCStr<i32> = wcstr!(i32, "Rust");
///
/// assert_eq!("Rust", RUST.to_string_lossy());
/// assert_eq!("Rust", ALSO_RUST.to_string_lossy());
/// ```
#[macro_export]
macro_rules! wcstr {
    ($ty:ident, $string:literal) => {
        unsafe {
            const STRING: &[$ty] = $crate::_wchar::wchz!($ty, $string);
            $crate::WCStr::from_slice_with_nul_unchecked(STRING)
        }
    };
    ($string:literal) => {
        unsafe {
            const STRING: &[$crate::WChar] = $crate::_wchar::wchz!($string);
            $crate::WCStr::from_slice_with_nul_unchecked(STRING)
        }
    };
}
