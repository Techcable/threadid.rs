//! Wraps [`std::thread::ThreadId`], providng faster lookup.
use core::borrow::Borrow;
use core::cell::Cell;
use core::ops::Deref;
use std::thread::ThreadId;

use equivalent::Equivalent;

fast_thread_local! {
    static STD_TID: Cell<Option<StdThreadId>> = Cell::new(None);
}

/// Wraps the [`std::thread::ThreadId`] type.
///
/// Using [`StdThreadId::current`] provides fast access to the current identifier.
/// It is significantly faster than using [`std::thread::current`],
/// which requires cloning a [`std::sync::Arc`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "std")))]
#[repr(transparent)]
pub struct StdThreadId(pub ThreadId);
impl StdThreadId {
    /// Lookup the [`std::thread::ThreadId`] of the current thread.
    #[inline]
    pub fn current() -> Self {
        STD_TID.with(|cell| match cell.get() {
            None => {
                let new_id = Self::acquire();
                cell.set(Some(new_id));
                new_id
            }
            Some(existing) => existing,
        })
    }
}
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "std")))]
// SAFETY: Wrapper around std::thread::ThreadId
unsafe impl crate::IThreadId for StdThreadId {
    #[inline]
    fn current() -> StdThreadId {
        <Self>::current()
    }
}
#[cfg(feature = "bytemuck")]
// SAFETY: We are #[repr(transparent)]
unsafe impl bytemuck::TransparentWrapper<ThreadId> for StdThreadId {}
// SAFETY: stdlib guarantees that threadid is unique
unsafe impl crate::IThreadId for ThreadId {
    #[inline]
    fn current() -> Self {
        StdThreadId::current().0
    }
}
impl StdThreadId {
    #[cold]
    fn acquire() -> StdThreadId {
        StdThreadId(std::thread::current().id())
    }
}
impl From<ThreadId> for StdThreadId {
    #[inline]
    fn from(value: ThreadId) -> Self {
        StdThreadId(value)
    }
}
impl From<StdThreadId> for ThreadId {
    #[inline]
    fn from(value: StdThreadId) -> Self {
        value.0
    }
}
impl Borrow<ThreadId> for StdThreadId {
    fn borrow(&self) -> &ThreadId {
        &self.0
    }
}
impl AsRef<ThreadId> for StdThreadId {
    #[inline]
    fn as_ref(&self) -> &ThreadId {
        &self.0
    }
}
impl Deref for StdThreadId {
    type Target = ThreadId;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl PartialEq<ThreadId> for StdThreadId {
    #[inline]
    fn eq(&self, other: &ThreadId) -> bool {
        self.0 == *other
    }
}
impl Equivalent<ThreadId> for StdThreadId {
    #[inline]
    fn equivalent(&self, key: &ThreadId) -> bool {
        *key == self.0
    }
}
#[cfg(feature = "slog")]
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "slog")))]
impl slog::Value for StdThreadId {
    fn serialize(&self, _record: &slog::Record, key: slog::Key, serializer: &mut dyn slog::Serializer) -> slog::Result {
        serializer.emit_arguments(key, &format_args!("{:?}", self))
    }
}
