use crate::reactor::*;
use derive_more::{Deref, DerefMut};
use futures::executor::block_on;
use futures::sink::{Sink, SinkExt};
use futures::task::{Context, Poll};
use pin_project::*;
use std::pin::Pin;

/// An adapter for types that implement [`Sink`] to behave as a [`Reactor`] (requires [`async`])
///
/// Returned by [`Reactor::from_sink`].
///
/// [`async`]: index.html#optional-features
/// [`Reactor::from_sink`]: trait.Reactor.html#method.from_sink
#[pin_project]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deref, DerefMut)]
pub struct AsyncReactor<T> {
    #[pin]
    sink: T,
}

impl<S, T, E> Reactor<S> for AsyncReactor<T>
where
    Self: for<'s> Sink<&'s S, Error = E> + Unpin,
{
    /// The reason why the state couldn't be sent through the sink.
    type Error = E;

    /// Sends an owned version of the state through the sink.
    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
        block_on(self.send(state))
    }
}

impl<S, T> Sink<&S> for AsyncReactor<T>
where
    S: Clone,
    T: Sink<S>,
{
    type Error = T::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, state: &S) -> Result<(), Self::Error> {
        self.project().sink.start_send(state.clone())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_close(cx)
    }
}

impl<S, E> dyn Reactor<S, Error = E> {
    /// Adapts any type that implements [`Sink`] as a [`Reactor`] (requires [`async`]).
    ///
    /// [`async`]: index.html#optional-features
    ///
    /// # Example
    /// ```rust
    /// use reducer::*;
    /// use futures::channel::mpsc::channel;
    /// use futures::executor::block_on_stream;
    /// use std::thread;
    ///
    /// let (tx, rx) = channel(0);
    /// let mut reactor = Reactor::<_, Error = _>::from_sink(tx);
    ///
    /// thread::spawn(move || {
    ///     reactor.react(&1);
    ///     reactor.react(&1);
    ///     reactor.react(&2);
    ///     reactor.react(&3);
    ///     reactor.react(&5);
    ///     reactor.react(&8);
    /// });
    ///
    /// assert_eq!(block_on_stream(rx).collect::<Vec<_>>(), vec![1, 1, 2, 3, 5, 8]);
    /// ```
    pub fn from_sink<T>(sink: T) -> AsyncReactor<T>
    where
        T: Sink<S, Error = E> + Unpin,
    {
        AsyncReactor { sink }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use proptest::prelude::*;
    use std::ops::*;

    #[test]
    fn deref() {
        let mut reactor = Reactor::<(), Error = ()>::from_sink(MockReactor::new());

        assert_eq!(reactor.deref() as *const _, &reactor.sink as *const _);
        assert_eq!(reactor.deref_mut() as *mut _, &mut reactor.sink as *mut _);
    }

    proptest! {
        #[test]
        fn react(state: u8, result: Result<(), u8>) {
            let mut mock = MockReactor::new();

            mock.expect_react()
                .with(eq(state.clone()))
                .times(1)
                .return_const(result);

            let mut reactor = Reactor::<_, Error = _>::from_sink(mock);
            assert_eq!(Reactor::react(&mut reactor, &state), result);
        }

        #[test]
        fn sink(state: u8, result: Result<(), u8>) {
            let mut mock = MockReactor::new();

            mock.expect_react()
                .with(eq(state.clone()))
                .times(1)
                .return_const(result);

            let mut reactor = Reactor::<_, Error = _>::from_sink(mock);
            assert_eq!(block_on(reactor.send(&state)), result);
            assert_eq!(block_on(reactor.close()), Ok(()));
        }
    }
}
