//! Defines [`UniqueThreadId`].

use core::num::NonZeroU64;

fast_thread_local! {
    #[cfg(not(all(feature = "nightly", feature = "std")))]
    static THREAD_ID: core::cell::Cell<Option<UniqueThreadId>> = core::cell::Cell::new(None);
}

/// A globally unique ThreadId.
///
/// Very similar to [`std::thread::ThreadId`],
/// except that it has less lookup overhead and gives access to the integer value.
/// If the `unique-wraps-std` feature is enabled,
/// the integer values are guaranteed to match the corresponding [`std::thread::ThreadId`].
/// If this feature is not enabled, they may not match.
///
/// While the current value is a [`core::num::NonZero`],
/// this may change in the future if other niche types like `NonMax` become stabilized.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(transparent)]
pub struct UniqueThreadId(NonZeroU64);
impl UniqueThreadId {
    /// Create a [`UniqueThreadId`] from an integer value.
    ///
    /// ## Safety
    /// The integer id must originate from calling [`UniqueThreadId::to_int`]
    /// in this same program execution.
    /// It is roughly equivalent to calling [`core::mem::transmute`].
    #[inline]
    pub unsafe fn from_int(x: u64) -> Self {
        // SAFETY: Caller guarantees that id is valid
        UniqueThreadId(unsafe { NonZeroU64::new_unchecked(x) })
    }

    /// Create a [`UniqueThreadId`] from a [`std::thread::ThreadId`].
    ///
    /// Requires the `unique-wrap-std` feature to be enabled,
    /// because otherwise the thread ids could differ.
    #[cfg(feature = "unique-wrap-std")]
    #[inline]
    pub fn from_std(id: impl Into<std::thread::ThreadId>) -> Self {
        // SAFETY: Enabling the feature guarantees ids are equivalent
        unsafe { Self::from_int(id.into().as_u64().get()) }
    }

    /// Convert a [`UniqueThreadId`] into an integer value.
    #[inline]
    pub fn to_int(&self) -> u64 {
        self.0.get()
    }

    #[cold]
    #[cfg(not(all(feature = "nightly", feature = "std")))]
    fn alloc() -> UniqueThreadId {
        use core::sync::atomic::Ordering;

        use portable_atomic::AtomicU64;
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_ID
            .fetch_update(Ordering::AcqRel, Ordering::Relaxed, |old_value| {
                old_value.checked_add(1)
            })
            .expect("id overflow");
        UniqueThreadId(NonZeroU64::new(id).unwrap())
    }

    /// Get the thread id of the currently executing thread.
    ///
    /// Will never be used by another thread,
    /// even if the current thread dies.
    ///
    /// If the `unique-wrap-std` feature is enabled,
    /// the id is guaranteed to match the corresponding [`std::thread::ThreadId`].
    #[inline]
    pub fn current() -> UniqueThreadId {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "std", feature = "nightly"))] {
                UniqueThreadId(crate::StdThreadId::current().0.as_u64())
            } else if #[cfg(feature = "unique-wrap-std")] {
                compile_error!("requires nightly + std")
            } else {
                THREAD_ID.with(|cell| {
                    match cell.get() {
                        None => {
                            let id = UniqueThreadId::alloc();
                            cell.set(Some(id));
                            id
                        }
                        Some(id) => id,
                    }
                })
            }
        }
    }
}
// SAFETY: Unique across all threads that have ever existed
unsafe impl crate::IThreadId for UniqueThreadId {
    #[inline]
    fn current() -> Self {
        <Self>::current()
    }
}
#[cfg(feature = "bytemuck")]
// SAFETY: Wraps a NonZero
unsafe impl bytemuck::ZeroableInOption for UniqueThreadId {}
#[cfg(feature = "bytemuck")]
// SAFETY: Wraps a NonZero
unsafe impl bytemuck::NoUninit for UniqueThreadId {}
impl From<UniqueThreadId> for u64 {
    #[inline]
    fn from(value: UniqueThreadId) -> Self {
        value.to_int()
    }
}
#[cfg(feature = "slog")]
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "slog")))]
impl slog::Value for UniqueThreadId {
    fn serialize(&self, _record: &slog::Record, key: slog::Key, serializer: &mut dyn slog::Serializer) -> slog::Result {
        serializer.emit_u64(key, self.to_int())
    }
}
