//! A predictable reactive framework for Rust apps.
//!
//! # Overview
//! Reducer is inspired by the
//! [Flux pattern](https://facebook.github.io/flux/docs/in-depth-overview.html), popular in
//! the JavaScript community as an effective idiom to build scalable and maintainable apps.
//!
//! The core mental model behind Reducer is the unidirectional data flow depicted below.
//!
//! ```notrust
//!                Reducer               |
//! +------------------------------------|-----------------+
//! |                                    |                 |
//! |    ----------        ---------     |      --------   |
//! +--> | Action | -----> | State | --- | ---> | View | --+
//!      ----------        ---------     |      --------
//!                                      |
//! ```
//!
//! The _view_, often a \[G\]UI, [dispatches](struct.Store.html#method.dispatch)
//! _actions_ on the [_store_](struct.Store.html), which in turn
//! [updates](trait.Reducer.html#tymethod.reduce) its internal state and
//! [notifies](trait.Reactor.html#tymethod.react) back the _view_.
//!
//! # Experimental Features
//!
//! The following cargo feature flags are available:
//! * `parallel` (depends on nightly Rust)
//!
//!     This feature flag takes advantage of the experimental support for specialization available
//!     on nightly Rust ([RFC 1210](https://github.com/rust-lang/rust/issues/31844)), to
//!     automatically parallelize tuples of
//!     [Sync](https://doc.rust-lang.org/nightly/std/marker/trait.Sync.html) /
//!     [Send](https://doc.rust-lang.org/nightly/std/marker/trait.Send.html) Reducers
//!     using [Rayon](https://crates.io/crates/rayon).
//!
//! * `async` (depends on nightly Rust)
//!
//!     Enables integration with [futures-rs](https://crates.io/crates/futures-preview).

#![cfg_attr(feature = "async", feature(async_await, await_macro, futures_api))]
#![cfg_attr(feature = "parallel", feature(specialization))]

#[cfg(feature = "async")]
extern crate futures;

#[cfg(feature = "parallel")]
extern crate rayon;

mod macros;
mod mock;

mod dispatcher;
mod reactor;
mod reducer;

pub use dispatcher::*;
pub use reactor::*;
pub use reducer::*;
