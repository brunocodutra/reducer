mod array;
#[cfg(feature = "alloc")]
mod boxed;
mod reference;
#[cfg(feature = "async")]
mod sink;
mod slice;
mod tuple;

#[cfg(feature = "async")]
pub use sink::AsyncReactor;

/// Trait for types that react to state transitions.
///
/// Reactors connect the _state_ to the _view_ components. They can implement arbitrary logic in
/// response to state transitions, but it's often better to think of Reactors as _channels_ that
/// transmit the current state to other parts of your application.
pub trait Reactor<S: ?Sized> {
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
    use mockall::{predicate::*, *};
    use proptest::prelude::*;
    use std::{boxed::Box, vec::Vec};

    mock! {
        pub(crate) Reactor<T: 'static, E: 'static> {
            fn id(&self) -> usize;
        }
        trait Reactor<T> {
            type Error = E;
            fn react(&mut self, state: &T) -> Result<(), E>;
        }
        trait Clone {
            fn clone(&self) -> Self;
        }
    }

    #[cfg(feature = "async")]
    use futures::Sink;

    #[cfg(feature = "async")]
    use std::{pin::Pin, task::Context, task::Poll};

    #[cfg(feature = "async")]
    #[cfg_attr(tarpaulin, skip)]
    impl<S: Unpin, E: Unpin> Sink<S> for MockReactor<S, E> {
        type Error = E;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, state: S) -> Result<(), Self::Error> {
            self.get_mut().react(&state)
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    proptest! {
        #[test]
        fn react(state: u8, result: Result<(), u8>) {
            let mut mock = MockReactor::new();

            mock.expect_react()
                .with(eq(state))
                .times(1)
                .return_const(result);

            let reactor: &mut dyn Reactor<_, Error = _> = &mut mock;
            assert_eq!(reactor.react(&state), result);
        }
    }
}

#[cfg(test)]
pub(crate) use self::tests::MockReactor;
