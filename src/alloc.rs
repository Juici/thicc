cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub use std::borrow::{Cow, ToOwned};
        pub use std::boxed::Box;
        pub use std::string::String;
        pub use std::vec::Vec;
    } else {
        extern crate alloc;

        pub use alloc::borrow::{Cow, ToOwned};
        pub use alloc::boxed::Box;
        pub use alloc::string::String;
        pub use alloc::vec::Vec;
    }
}
