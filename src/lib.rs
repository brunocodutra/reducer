//! A predictable reactive framework for Rust apps.
//!
//! Reducer is a platform for reactive programming in Rust that can be used to manage the state of
//! any kind of application. It shines when used to drive graphical user interfaces and integrates
//! well with both immediate mode and retained mode GUI frameworks
//! ([check out the examples](https://github.com/brunocodutra/reducer/tree/master/examples)).
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
//! +--> | Action | -----> | Store | --- | ---> | View | --+
//!      ----------        ---------     |      --------
//!                                      |
//! ```
//!
//! The _view_, often a \[G\]UI, [dispatches](struct.Store.html#method.dispatch)
//! _actions_ on the [_store_](struct.Store.html), which in turn
//! [updates](trait.Reducer.html#method.reduce) its internal state and
//! [notifies](trait.Subscriber.html#method.notify) back the _view_.
//!
//! # Usage
//! ```rust
//! extern crate reducer;
//!
//! use reducer::*;
//! use std::io::{self, Write};
//!
//! struct Calculator(i32);
//!
//! enum Operation {
//!     Add(i32),
//!     Sub(i32),
//!     Mul(i32),
//!     Div(i32),
//! }
//!
//! use Operation::*;
//!
//! impl Reducer for Calculator {
//!     type Action = Operation;
//!     fn reduce(&mut self, op: Self::Action) {
//!         match op {
//!             Add(x) => self.0 += x,
//!             Sub(x) => self.0 -= x,
//!             Mul(x) => self.0 *= x,
//!             Div(x) => self.0 /= x,
//!         }
//!     }
//! }
//!
//! struct Reactor;
//!
//! impl Subscriber<Calculator> for Reactor {
//!     type Error = io::Error;
//!     fn notify(&self, state: &Calculator) -> io::Result<()> {
//!         io::stdout().write_fmt(format_args!("{}\n", state.0))
//!     }
//! }
//!
//! fn main() {
//!     let mut store = Store::new(Calculator(0), Reactor);
//!
//!     store.dispatch(Add(5)).unwrap(); // prints "5"
//!     store.dispatch(Mul(3)).unwrap(); // prints "15"
//!     store.dispatch(Sub(1)).unwrap(); // prints "14"
//!     store.dispatch(Div(7)).unwrap(); // prints "2"
//! }
//! ```

mod reducer;
mod store;
mod subscriber;

pub use reducer::*;
pub use store::*;
pub use subscriber::*;
