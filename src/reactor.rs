mod array;
mod boxed;
mod option;
mod reference;
mod sender;
mod slice;
mod tuple;

/// Trait for types that react to state transitions.
///
/// Reactors connect the _state_ to the _view_ components. They can implement arbitrary logic in
/// response to state transitions, but it's often better to think of Reactors as _channels_ that
/// transmit the current state to other parts of your application.
///
/// # Reactor as a Data Channel
/// For GUI applications, it is a good practice to have a separate thread dedicated to rendering.
/// To help wiring up the Flux pattern in such multi-threaded scenarios, Reactor is implemented
/// for [`mpsc::Sender`](trait.Reactor.html#impl-Reactor<S>) out of the box.
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
    /// The result of reacting to `S`.
    type Output;

    /// Reacts to `S` and produces `Self::Output`.
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
    ///     type Output = io::Result<()>;
    ///     fn react(&self, state: &T) -> Self::Output {
    ///         io::stdout().write_fmt(format_args!("{:?}\n", state))
    ///     }
    /// }
    /// ```
    fn react(&self, state: &S) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn react(states: Vec<u8>) {
            let reactor: &Reactor<_, Output = _> = &MockReactor::default();

            for state in states {
                assert_eq!(reactor.react(&state), state);
            }
        }
    }
}
