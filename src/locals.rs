macro_rules! fast_thread_local {
    ($($(#[$field_attr:meta])* static $var:ident: $tp:ty = $init:expr;)*) => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "nightly")] {
                $(
                $(#[$field_attr])*
                #[thread_local]
                static $var: $crate::locals::nightly::NightlyLocalKey<$tp> = $crate::locals::nightly::NightlyLocalKey::new($init);)*
            } else if #[cfg(feature = "std")] {
                std::thread_local! {
                    $(
                    $(#[$field_attr])*
                    static $var: $tp = const { $init };)*
                }
            } else {
                compile_error!("Either the `std` or `nightly` feature must be enabled");
            }
        }
    };
}

#[cfg(feature = "nightly")]
#[cfg_attr(not(feature = "std"), allow(dead_code))]
pub mod nightly {
    pub struct NightlyLocalKey<T> {
        value: T,
    }
    impl<T: 'static> NightlyLocalKey<T> {
        pub const fn new(value: T) -> NightlyLocalKey<T> {
            NightlyLocalKey { value }
        }
        #[inline]
        pub fn with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
            func(&self.value)
        }
        #[inline]
        pub fn try_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> Result<R, AccessError> {
            Ok(func(&self.value))
        }
    }
    type AccessError = core::convert::Infallible;
}
