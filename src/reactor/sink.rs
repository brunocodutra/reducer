use crate::reactor::*;
use derive_more::{Deref, DerefMut};
use futures::executor::block_on;
use futures::sink::{Sink, SinkExt};
use pin_project::*;
use std::task::{Context, Poll};
use std::{borrow::ToOwned, pin::Pin};

/// An adapter for types that implement [`Sink`] to behave as a [`Reactor`] (requires [`async`])
///
/// Returned by [`Reactor::from_sink`] and [`Reactor::from_sink_with`].
///
/// [`async`]: index.html#optional-features
/// [`Reactor::from_sink`]: trait.Reactor.html#method.from_sink
/// [`Reactor::from_sink_with`]: trait.Reactor.html#method.from_sink_with
#[pin_project(project = AsyncReactorProjection)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deref, DerefMut)]
pub struct AsyncReactor<T, F> {
    #[pin]
    #[deref]
    #[deref_mut]
    sink: T,
    with: F,
}

impl<S, T, F, E> Reactor<S> for AsyncReactor<T, F>
where
    S: ?Sized,
    Self: for<'s> Sink<&'s S, Error = E> + Unpin,
{
    /// The reason why the state couldn't be sent through the sink.
    type Error = E;

    /// Sends an owned version of the state through the sink.
    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
        block_on(self.send(state))
    }
}

impl<S, T, F, O> Sink<&S> for AsyncReactor<T, F>
where
    S: ?Sized,
    T: Sink<O>,
    F: for<'s> Fn(&'s S) -> O,
{
    type Error = T::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, state: &S) -> Result<(), Self::Error> {
        let AsyncReactorProjection { sink, with } = self.project();
        sink.start_send(with(state))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_close(cx)
    }
}

impl<S, E> dyn Reactor<S, Error = E>
where
    S: ?Sized,
{
    /// Adapts any type that implements [`Sink`] as a [`Reactor`] (requires [`async`]).
    ///
    /// This function is equivalent to
    /// [`Reactor::<S, Error = E>::from_sink_with(sink, S::to_owned)`][from_sink_with].
    ///
    /// [`async`]: index.html#optional-features
    /// [from_sink_with]: trait.Reactor.html#method.from_sink_with
    pub fn from_sink<T>(sink: T) -> AsyncReactor<T, fn(&S) -> S::Owned>
    where
        S: ToOwned,
        T: Sink<S::Owned, Error = E> + Unpin,
    {
        Reactor::<S, Error = E>::from_sink_with(sink, S::to_owned)
    }

    /// Adapts any type that implements [`Sink`] as a [`Reactor`] (requires [`async`]).
    ///
    /// [`async`]: index.html#optional-features
    ///
    /// # Example
    /// ```rust
    /// use reducer::*;
    /// use futures::channel::mpsc::channel;
    /// use futures::executor::block_on_stream;
    /// use std::{thread, string::ToString};
    ///
    /// let (tx, rx) = channel(0);
    /// let mut reactor = Reactor::<_, Error = _>::from_sink_with(tx, ToString::to_string);
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
    /// assert_eq!(block_on_stream(rx).collect::<String>(), "112358".to_string());
    /// ```
    pub fn from_sink_with<T, F, O>(sink: T, with: F) -> AsyncReactor<T, F>
    where
        T: Sink<O, Error = E> + Unpin,
        F: for<'s> Fn(&'s S) -> O,
    {
        AsyncReactor { sink, with }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use proptest::prelude::*;
    use std::{ops::*, string::String};

    #[test]
    fn deref() {
        let mut reactor = Reactor::<(), Error = ()>::from_sink(MockReactor::new());

        assert_eq!(reactor.deref() as *const _, &reactor.sink as *const _);
        assert_eq!(reactor.deref_mut() as *mut _, &mut reactor.sink as *mut _);
    }

    proptest! {
        #[test]
        fn react(state: String, result: Result<(), u8>) {
            let mut mock = MockReactor::new();

            mock.expect_react()
                .with(eq(state.clone()))
                .times(1)
                .return_const(result);

            let mut reactor = Reactor::<str, Error = _>::from_sink(mock);
            assert_eq!(Reactor::react(&mut reactor, &state), result);
        }

        #[test]
        fn sink(state: String, result: Result<(), u8>) {
            let mut mock = MockReactor::new();

            mock.expect_react()
                .with(eq(state.clone()))
                .times(1)
                .return_const(result);

            let mut reactor = Reactor::<str, Error = _>::from_sink(mock);
            assert_eq!(block_on(reactor.send(&state)), result);
            assert_eq!(block_on(reactor.close()), Ok(()));
        }
    }
}
