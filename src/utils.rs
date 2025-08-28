#[cfg(feature = "parking_lot")]
pub mod sync {
    pub use parking_lot::{Mutex, MutexGuard};
}

/// Wrappers around the standard synchronization primitives which do not poison.
#[cfg(all(feature = "std", not(feature = "parking_lot")))]
pub mod sync {
    pub use std::sync::MutexGuard;
    use std::sync::PoisonError;

    pub struct Mutex<T>(pub std::sync::Mutex<T>);
    impl<T> Mutex<T> {
        /// Create a new `Mutex`.
        pub const fn new(value: T) -> Self {
            Mutex(std::sync::Mutex::new(value))
        }
        #[inline]
        pub fn lock(&self) -> MutexGuard<'_, T> {
            self.0.lock().unwrap_or_else(PoisonError::into_inner)
        }
    }
}

#[cfg_attr(not(feature = "std"), allow(unused_imports))]
pub use self::cell::OnceCell;
#[cfg_attr(not(feature = "std"), allow(dead_code))]
mod cell {
    use core::cell::UnsafeCell;

    /// A version of [`std::cell::OnceCell`] that supports our MSRV.
    ///
    /// NOTE: The `once_lock` crate has a newer MSRV than we do,
    /// and gives rather poor MSRV guarantees.
    pub struct OnceCell<T> {
        value: UnsafeCell<Option<T>>,
    }
    impl<T> OnceCell<T> {
        #[inline]
        pub const fn new() -> Self {
            OnceCell {
                value: UnsafeCell::new(None),
            }
        }

        #[inline]
        pub fn get(&self) -> Option<&'_ T> {
            // SAFETY: Guaranteed to only be initialized once
            unsafe {
                let x: &Option<T> = &*self.value.get();
                x.as_ref()
            }
        }

        #[inline]
        pub fn set(&self, new_value: T) -> Result<(), &'_ T> {
            // SAFETY: Upholds the invariant that initialization is only done once
            // Cannot be called from multiple threads, since !Sync,
            // and cannot be called recursively because there is no callback
            unsafe {
                let x: &mut Option<T> = &mut *self.value.get();
                match *x {
                    Some(ref existing) => Err(existing),
                    None => {
                        *x = Some(new_value);
                        Ok(())
                    }
                }
            }
        }
    }
    // SAFETY: Fine to send because old thread loses access
    unsafe impl<T> Send for OnceCell<T> {}
}
macro_rules! simple_serde_serialize {
    ($target:ident, |$this:ident| $to_inner:expr) => {
        #[cfg(feature = "serde")]
        impl serde::Serialize for $target {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let $this = self;
                let value = $to_inner;
                serde::Serialize::serialize(&value, serializer)
            }
        }
    };
}
