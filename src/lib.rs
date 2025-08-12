//! Fast and flexible thread identifiers.
//!
//! The main reason this crate exists is performance.
//! Retrieving any of these ids is 30x faster than calling [`std::thread::current`],
//! which calls [`Arc::clone`] internally.
//! The three types of ids [`StdThreadId`], [`UniqueThreadId`], and [`LiveThreadId`]
//! are roughly equivalent in lookup performance
//!
//! The other reason this crate exists is flexibility.
//! Using [`LiveThreadId`] will aggressively reuse thread ids to minimize the integer value of ids,
//! making it useful as a key in a vector of live threads.
//! Using [`debug::DebugThreadId`] is a convenience wrapper which displays [`std::thread::Thread::name`]
//! where possible, in addition to using [`UniqueThreadId`] as a fallback when the thread is unnamed.
#![cfg_attr(
    feature = "nightly",
    feature(
        // using #[thread_local] works on #[no_std],
        // and could be faster in some cases
        thread_local,
    )
)]
#![cfg_attr(
    all(feature = "nightly", feature = "std"),
    feature(
        // used to make UniqueThreadId match std::thread::ThreadId
        thread_id_value,
    )
)]
#![cfg_attr(feature = "nightly-docs", feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    unsafe_op_in_unsafe_fn,
    missing_docs,
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::alloc_instead_of_core,
    clippy::undocumented_unsafe_blocks
)]
#![warn(clippy::multiple_unsafe_ops_per_block, clippy::pedantic)]
#![allow(
    // stylistic
    clippy::single_match_else,
)]

#[cfg(not(any(feature = "nightly", feature = "std")))]
compile_error!("Either the `nightly` or `std` feature must be enabled for this crate to work");

#[cfg(all(feature = "unique-wrap-std", not(feature = "nightly")))]
compile_error!("The `unique-wrap-std` feature currently requires the `nightly` feature to be enabled");

#[cfg(feature = "alloc")]
extern crate alloc;

use core::fmt::Debug;
use core::hash::Hash;
#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "std")))]
pub use std::StdThreadId;

#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "std")))]
pub use live::LiveThreadId;
pub use unique::UniqueThreadId;

#[macro_use]
mod locals;
#[cfg(feature = "std")]
pub mod debug;
#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "std")))]
pub mod live;
#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "std")))]
pub mod std;
pub mod unique;
mod utils;

/// Defines methods common to all thread ids.
///
/// ## Safety
/// Ids are guaranteed to differ across live threads for [`LiveThreadId`],
/// and among all threads that have ever existed for [`UniqueThreadId`] and [`StdThreadId`].
pub unsafe trait IThreadId: Copy + Eq + Hash + Debug + sealed::Sealed {
    /// Get the id of the currently executing thread.
    ///
    /// May panic if called from a thread destructor.
    fn current() -> Self;
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for crate::UniqueThreadId {}
    #[cfg(feature = "std")]
    impl Sealed for crate::LiveThreadId {}
    #[cfg(feature = "std")]
    impl Sealed for crate::StdThreadId {}
    #[cfg(feature = "std")]
    impl Sealed for std::thread::ThreadId {}
}

/// Get the id of the current thread.
///
/// Convenience method for calling [`IThreadId::current`].
#[inline]
#[must_use]
pub fn current<T: IThreadId>() -> T {
    T::current()
}
