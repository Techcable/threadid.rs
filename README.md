# threadid.rs
<!-- cargo-rdme start -->

Fast and flexible thread identifiers.

The main reason this crate exists is performance.
Retrieving any of these ids is 30x faster than calling
[`std::thread::current`][fn@::std::thread::current],
which calls [`Arc::clone`][fn@::std::sync::Arc::clone] internally.
The three types of ids [`StdThreadId`], [`UniqueThreadId`], and [`LiveThreadId`]
are roughly equivalent in lookup performance

The other reason this crate exists is flexibility.
Using [`LiveThreadId`] will aggressively reuse thread ids to minimize the integer value of ids,
making it useful as a key in a vector of live threads.
Using [`debug::DebugThreadId`] is a convenience wrapper which displays [`std::thread::Thread::name`](https://doc.rust-lang.org/stable/std/thread/struct.Thread.html#method.name)
where possible, in addition to using [`UniqueThreadId`] as a fallback when the thread is unnamed.

## Similar Crates
The crate name and functionality is similar to that of the `thread-id` crate,
which simply wraps OS primitives (pthreads/windows).
Make sure you are using the right crate.
They have very different APIs.

<!-- cargo-rdme end -->

[fn@::std::thread::current]: https://doc.rust-lang.org/stable/std/thread/fn.current.html
[fn@::std::sync::Arc::clone]: https://doc.rust-lang.org/std/sync/struct.Arc.html#method.clone
[`StdThreadId`]: https://docs.rs/threadid/latest/threadid/std/struct.StdThreadId.html
[`UniqueThreadId`]: https://docs.rs/threadid/latest/threadid/unique/struct.UniqueThreadId.html
[`LiveThreadId`]: https://docs.rs/threadid/latest/threadid/live/struct.LiveThreadId.html
[`debug::DebugThreadId`]: https://docs.rs/threadid/latest/threadid/debug/struct.DebugThreadId.html


## License
Licensed under either the [Apache 2.0 License](./LICENSE-APACHE.txt) or [MIT License](./LICENSE-MIT.txt) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
