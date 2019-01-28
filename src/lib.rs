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
//! * `async` (depends on nightly Rust)
//!
//!     Enables integration with [futures-rs](https://crates.io/crates/futures-preview).

#![cfg_attr(
    feature = "async",
    feature(async_await, await_macro, futures_api, existential_type)
)]

mod macros;
mod mock;

mod dispatcher;
mod reactor;
mod reducer;

pub use crate::dispatcher::*;
pub use crate::reactor::*;
pub use crate::reducer::*;
