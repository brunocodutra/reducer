mod array;
#[cfg(feature = "std")]
mod boxed;
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
