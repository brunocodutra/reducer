mod any;
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
/// Subscribers connect the _state_ to the _view_ components. They can implement arbitrary logic in
/// response to state transitions, but it's often better to think of Subscribers as _channels_ that
/// transmit the current state to other parts of your application.
///
/// # Subscriber as a Data Channel
/// For GUI applications, it is a good practice to have a separate thread dedicated to rendering.
/// To help wiring up the Flux pattern in such multi-threaded scenarios, Subscriber is implemented
/// for [`mpsc::Sender`](trait.Subscriber.html#impl-Subscriber<S>) out of the box.
///
/// ## Example
/// ```rust
/// use reducer::Subscriber;
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
///     // Remember that tx implements Subscriber.
///     while let Ok(()) = tx.notify(&countdown) {
///         // Update the state.
///         countdown -= 1;
///     }
/// }
/// ```
pub trait Subscriber<S> {
    /// The type returned if the Subscriber fails to react to a state transition.
    type Error: Debug;

    /// Reacts to a state transition or returns `Err(Self::Error)` in case of failure.
    fn notify(&self, state: &S) -> Result<(), Self::Error>;
}

pub use self::any::AnySubscriber;

#[cfg(test)]
pub use self::mock::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notify() {
        let mock = MockSubscriber::default();

        {
            let sbc: &Subscriber<_, Error = _> = &mock;

            assert!(sbc.notify(&5).is_ok());
            assert!(sbc.notify(&1).is_ok());
            assert!(sbc.notify(&3).is_ok());
        }

        assert_eq!(mock, MockSubscriber::new(vec![5, 1, 3]));
    }
}
