mod array;
mod mock;
mod option;
mod reference;
mod sender;
mod slice;
mod tuple;

use std::fmt::Debug;

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
/// ```rust
/// use reducer::Reactor;
///
/// fn main() {
///     // Create a channel for the current state.
///     let (tx, rx) = std::sync::mpsc::channel();
///
///     // Start the rendering thread.
///     std::thread::spawn(move || {
///         println!("Preparing to launch");
///         loop {
///             // Render the current state to the screen.
///             match rx.recv() {
///                 Ok(10) => println!("T-10 seconds - Activate main engine hydrogen burnoff system."),
///                 Ok(6) => println!("T-6 seconds - Main engine start."),
///                 Ok(0) => println!("T-0 seconds - Solid rocket booster ignition and liftoff!"),
///                 Ok(countdown) if countdown > 0 => println!("T-{} seconds", countdown),
///                 _ => break,
///             }
///         }
///     });
///
///     // Set-up the initial state.
///     let mut countdown = 10;
///
///     // Remember that tx implements Reactor.
///     while let Ok(()) = tx.react(&countdown) {
///         // Update the state.
///         countdown -= 1;
///     }
/// }
/// ```
pub trait Reactor<S> {
    /// The type returned if the Reactor fails to react to a state transition.
    type Error: Debug;

    /// Reacts to a state transition or returns `Err(Self::Error)` in case of failure.
    fn react(&self, state: &S) -> Result<(), Self::Error>;
}

#[cfg(test)]
pub use self::mock::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn react() {
        let mock = MockReactor::default();

        {
            let sbc: &Reactor<_, Error = _> = &mock;

            assert!(sbc.react(&5).is_ok());
            assert!(sbc.react(&1).is_ok());
            assert!(sbc.react(&3).is_ok());
        }

        assert_eq!(mock, MockReactor::new(vec![5, 1, 3]));
    }
}
