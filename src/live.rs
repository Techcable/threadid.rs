//! Defines a [`LiveThreadId`].
//!
//! The implementation is inspired by the implementation of thread ids in the [`thread_local`] crate:
//! <https://github.com/Amanieu/thread_local-rs/blob/8958483/src/thread_id.rs>

use alloc::collections::BinaryHeap;
use core::cell::Cell;
use core::fmt::{Debug, Formatter};

use nonmax::NonMaxUsize;

use crate::utils::OnceCell;
use crate::utils::sync::{Mutex, MutexGuard};

/// Identifies a live thread.
///
/// Unlike [`UniqueThreadId`](crate::UniqueThreadId) or [`std::thread::ThreadId`],
/// the id may be reused once a thread dies.
///
/// The implementation will try to minimize the integer value of the ids
/// by aggressively reusing ids whenever possible.
/// This makes it is sensible to use the id to index a vector.
///
/// It is guaranteed that `Option<LiveThreadId>` has the same representation as `LiveThreadId`.
/// Currently [`LiveThreadId::to_int`] can be zero, reducing wasted indexes.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[must_use]
#[repr(transparent)]
pub struct LiveThreadId {
    index: NonMaxUsize,
}
impl LiveThreadId {
    /// Get the id of the currently executing thread.
    ///
    /// Ids will be reused once a thread dies.
    ///
    /// May panic if called from a thread destructor.
    #[inline]
    pub fn current() -> Self {
        LIVE_ID.with(|cell| match cell.get() {
            Some(existing) => existing,
            None => {
                let new_id = Self::alloc();
                cell.set(Some(new_id));
                new_id
            }
        })
    }
}
// SAFETY: Differs across live threads
unsafe impl crate::IThreadId for LiveThreadId {
    #[inline]
    fn current() -> Self {
        <Self>::current()
    }
}
impl Debug for LiveThreadId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("LiveThreadId").field(&self.index()).finish()
    }
}
impl LiveThreadId {
    #[cold]
    fn alloc() -> LiveThreadId {
        GUARD
            .try_with(|cell| {
                assert!(cell.get().is_none(), "already initialized");
            })
            .unwrap_or_else(|_| panic!("thread already destroyed"));
        let mut alloc = ThreadIdAllocator::lock();
        let alloc = ThreadIdAllocator::lazy_init(&mut alloc);
        let new_id = if let Some(existing) = alloc.free_list.pop() {
            LiveThreadId { index: existing.0 }
        } else {
            let next_id = alloc.next_id.get();
            alloc.next_id.set(
                next_id
                    .get()
                    .checked_add(1)
                    .and_then(NonMaxUsize::new)
                    .expect("LiveThreadId overflowed a usize"),
            );
            LiveThreadId { index: next_id }
        };
        GUARD.with(|cell| {
            cell.set(ThreadGuard { id: new_id })
                .unwrap_or_else(|_| panic!("already initialized"));
        });
        new_id
    }

    /// Get the integer value of this thread id.
    ///
    /// This is an alias for [`Self::to_int`].
    #[inline]
    #[must_use]
    pub fn index(self) -> usize {
        self.index.get()
    }

    /// Get the integer value of this thread id.
    ///
    /// Note that zero is currently a valid index, although `usize::MAX` is not.
    /// The implementation will attempt to minimize the integer value of this id.
    ///
    /// The index is guaranteed to fit into a `usize`,
    /// because there cannot be more live threads then memory addresses.
    #[inline]
    #[must_use]
    pub fn to_int(self) -> usize {
        self.index.get()
    }
}
simple_serde_serialize!(LiveThreadId, |this| this.to_int());
#[cfg(feature = "bytemuck")]
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "bytemuck")))]
// SAFETY: We wrap a NonMax, which has the same niche as NonZero
unsafe impl bytemuck::ZeroableInOption for LiveThreadId {}
#[cfg(feature = "bytemuck")]
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "bytemuck")))]
// SAFETY: A NonMax is equivalent to a NonZero
unsafe impl bytemuck::NoUninit for LiveThreadId {}
#[cfg(feature = "slog")]
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "slog")))]
impl slog::Value for LiveThreadId {
    fn serialize(&self, _record: &slog::Record, key: slog::Key, serializer: &mut dyn slog::Serializer) -> slog::Result {
        serializer.emit_arguments(key, &format_args!("{self:?}"))
    }
}

fast_thread_local! {
    static LIVE_ID: Cell<Option<LiveThreadId>> = Cell::new(None);
}
std::thread_local! {
    /// Runs a destructor to reuse a thread id
    static GUARD: OnceCell<ThreadGuard> = const { OnceCell::new() };
}
struct ThreadGuard {
    id: LiveThreadId,
}
impl Drop for ThreadGuard {
    fn drop(&mut self) {
        let _ = LIVE_ID.try_with(|id| id.set(None));
        let mut alloc = ThreadIdAllocator::lock();
        let alloc = ThreadIdAllocator::lazy_init(&mut alloc);
        alloc.free_list.push(core::cmp::Reverse(self.id.index));
    }
}

/// Reuses the thread ids of dead threads.
static ALLOCATOR: Mutex<Option<ThreadIdAllocator>> = Mutex::new(None);

struct ThreadIdAllocator {
    next_id: Cell<NonMaxUsize>,
    free_list: BinaryHeap<core::cmp::Reverse<NonMaxUsize>>,
}
impl ThreadIdAllocator {
    #[inline]
    fn lock() -> MutexGuard<'static, Option<ThreadIdAllocator>> {
        ALLOCATOR.lock()
    }
    #[inline]
    fn lazy_init<'a>(lock: &'a mut MutexGuard<'static, Option<ThreadIdAllocator>>) -> &'a mut ThreadIdAllocator {
        #[cold]
        fn init() -> ThreadIdAllocator {
            ThreadIdAllocator {
                free_list: BinaryHeap::new(),
                next_id: Cell::new(NonMaxUsize::ZERO),
            }
        }
        lock.get_or_insert_with(init)
    }
}
