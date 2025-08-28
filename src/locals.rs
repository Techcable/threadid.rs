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
                $(
                static $var: $crate::locals::dummy::DummyLocalKey<$tp> = $crate::locals::dummy::DummyLocalKey::new($init);
                )*
            }
        }
    };
}

/// Version of [`std::thread::LocalKey`] using the nightly `#[thread_local]` attribute.
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
        #[allow(clippy::unnecessary_wraps)]
        pub fn try_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> Result<R, AccessError> {
            Ok(func(&self.value))
        }
    }
    type AccessError = core::convert::Infallible;
}

/// Dummy version of [`std::thread::LocalKey`] to avoid duplicate compilation errors.
#[cfg(not(any(feature = "nightly", feature = "std")))]
pub mod dummy {
    use core::mem::ManuallyDrop;

    pub struct DummyLocalKey<T> {
        _value: ManuallyDrop<T>,
    }
    // always Sync because we don't give any access
    unsafe impl<T> Sync for DummyLocalKey<T> {}
    impl<T: 'static> DummyLocalKey<T> {
        pub const fn new(value: T) -> Self {
            DummyLocalKey {
                _value: ManuallyDrop::new(value),
            }
        }
        #[inline]
        pub fn with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
            let _ = func;
            unimplemented!("thread local unsupported")
        }
        #[inline]
        #[allow(clippy::unnecessary_wraps)]
        pub fn try_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> Result<R, AccessError> {
            let _ = func;
            Err("thread local unsupported")
        }
    }
    type AccessError = &'static str;
}
