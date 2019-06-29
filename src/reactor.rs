mod array;
mod boxed;
mod mpsc;
mod reference;
#[cfg(feature = "async")]
mod sink;
mod slice;
mod tuple;

/// Trait for types that react to state transitions.
///
/// Reactors connect the _state_ to the _view_ components. They can implement arbitrary logic in
/// response to state transitions, but it's often better to think of Reactors as _channels_ that
/// transmit the current state to other parts of your application.
///
/// # Reactor as a Data Channel
///
/// For GUI applications, it is a good practice to have a separate thread dedicated to rendering.
/// To help wiring up the Flux pattern in such multi-threaded scenarios, Reactor is implemented
/// for [`mpsc::Sender`] out of the box.
///
/// ## Example
///
/// ```rust
/// use reducer::*;
///
/// fn main() {
///     // Create a channel for the current state.
///     let (tx, rx) = std::sync::mpsc::channel();
///
///     // Start the rendering thread.
///     std::thread::spawn(move || {
///         while let Ok(Countdown(t)) = rx.recv() {
///             // Render the current state to the screen.
///             match t {
///                 6 => println!("T-6 seconds - Main engine start."),
///                 0 => println!("T-0 seconds - Solid rocket booster ignition and liftoff!"),
///                 t if t > 0 => println!("T-{} seconds", t),
///                 _ => break,
///             }
///         }
///     });
///
///     #[derive(Clone)]
///     struct Countdown(i32);
///
///     struct Tick;
///     impl Reducer<Tick> for Countdown {
///         fn reduce(&mut self, _: Tick) {
///             self.0 -= 1;
///         }
///     }
///
///     // Set-up the initial state.
///     let mut store = Store::new(Countdown(10), tx);
///
///     // Count down to liftoff!
///     while let Ok(()) = store.dispatch(Tick) {}
/// }
/// ```
pub trait Reactor<S> {
    /// The type returned if the Reactor fails.
    type Error;

    /// Reacts to an update to `S`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use reducer::*;
    /// use std::fmt::Debug;
    /// use std::io::{self, Write};
    ///
    /// struct Console;
    ///
    /// impl<T: Debug> Reactor<T> for Console {
    ///     type Error = io::Error;
    ///     fn react(&mut self, state: &T) -> io::Result<()> {
    ///         io::stdout().write_fmt(format_args!("{:?}\n", state))
    ///     }
    /// }
    /// ```
    fn react(&mut self, state: &S) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn react(states: Vec<u8>) {
            let mut mock = Mock::<_>::default();

            for (i, state) in states.iter().enumerate() {
                let reactor: &mut dyn Reactor<_, Error = _> = &mut mock;
                assert_eq!(reactor.react(state), Ok(()));
                assert_eq!(mock.calls(), &states[0..=i]);
            }
        }
    }
}
