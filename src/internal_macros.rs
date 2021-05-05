#![allow(unused_macros)]

/// Asserts that a condition is true at compile time.
macro_rules! static_assert {
    ($cond:expr $(,)?) => {
        const _: [(); 0 - !{
            const COND: bool = $cond;
            COND
        } as usize] = [];
    };
}

/// Asserts that a type implements traits at compile time.
macro_rules! assert_impls {
    ($($impls:tt)*) => {
        static_assert!(::impls::impls!($($impls)*));
    };
}
