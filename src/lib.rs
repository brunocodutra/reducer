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
//! [updates](trait.Reducer.html#tymethod.reduce) its internal state and
//! [notifies](trait.Reactor.html#tymethod.react) back the _view_.
//!
//! # Usage
//! ```rust
//! extern crate reducer;
//!
//! use reducer::*;
//! use std::io::{self, Write};
//!
//! // The state of your app.
//! struct Calculator(i32);
//!
//! // Actions the user can trigger.
//! struct Add(i32);
//! struct Sub(i32);
//! struct Mul(i32);
//! struct Div(i32);
//!
//! impl Reducer<Add> for Calculator {
//!     fn reduce(&mut self, Add(x): Add) {
//!         self.0 += x;
//!     }
//! }
//!
//! impl Reducer<Sub> for Calculator {
//!     fn reduce(&mut self, Sub(x): Sub) {
//!         self.0 -= x;
//!     }
//! }
//!
//! impl Reducer<Mul> for Calculator {
//!     fn reduce(&mut self, Mul(x): Mul) {
//!         self.0 *= x;
//!     }
//! }
//!
//! impl Reducer<Div> for Calculator {
//!     fn reduce(&mut self, Div(x): Div) {
//!         self.0 /= x;
//!     }
//! }
//!
//! // The user interface.
//! struct Ui;
//!
//! impl Reactor<Calculator> for Ui {
//!     type Error = io::Error;
//!     fn react(&self, state: &Calculator) -> io::Result<()> {
//!         io::stdout().write_fmt(format_args!("{}\n", state.0))
//!     }
//! }
//!
//! fn main() {
//!     let mut store = Store::new(Calculator(0), Ui);
//!
//!     store.dispatch(Add(5)).unwrap(); // displays "5"
//!     store.dispatch(Mul(3)).unwrap(); // displays "15"
//!     store.dispatch(Sub(1)).unwrap(); // displays "14"
//!     store.dispatch(Div(7)).unwrap(); // displays "2"
//! }
//! ```

mod reactor;
mod reducer;
mod store;

pub use reactor::*;
pub use reducer::*;
pub use store::*;
