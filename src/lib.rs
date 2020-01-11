//! A predictable reactive framework for Rust apps.
//!
//! # Overview
//! Reducer is inspired by the
//! [Flux pattern], popular in the JavaScript community as an effective idiom to build
//! scalable and maintainable apps.
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
//! The _view_, often a \[G\]UI, [dispatches] _actions_ on the [_store_], which in turn
//! [updates] its internal state and [notifies] back the _view_.
//!
//! [Flux pattern]: https://facebook.github.io/flux/docs/in-depth-overview.html
//! [dispatches]: struct.Store.html#method.dispatch
//! [_store_]: struct.Store.html
//! [updates]: trait.Reducer.html#tymethod.reduce
//! [notifies]: trait.Reactor.html#tymethod.react
//!
//! # Optional Features
//!
//! * `std` (enabled by default)
//!
//!     You can disable this feature to use Reducer in a `#[no_std]` environment.
//!
//! * `async` (enabled by default, requires `std`)
//!
//!     Enables integration with [futures-rs](https://crates.io/crates/futures).

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[macro_use]
extern crate std;

mod macros;

mod dispatcher;
mod reactor;
mod reducer;

pub use crate::dispatcher::*;
pub use crate::reactor::*;
pub use crate::reducer::*;
