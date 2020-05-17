use crate::reactor::*;
use futures::executor::block_on;
use futures::sink::{Sink, SinkExt};
use futures::task::{Context, Poll};
use pin_project::*;
use std::{borrow::ToOwned, ops::*, pin::Pin};

/// An adapter for types that implement [`Sink`] to behave as a [`Reactor`] (requires [`async`])
///
/// Returned by [`Reactor::from_sink`].
///
/// [`async`]: index.html#optional-features
/// [`Reactor::from_sink`]: trait.Reactor.html#method.from_sink
#[pin_project]
pub struct SinkAsReactor<T> {
    #[pin]
    sink: T,
}

impl<T> Deref for SinkAsReactor<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.sink
    }
}

impl<T> DerefMut for SinkAsReactor<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sink
    }
}

impl<S, T> Reactor<S> for SinkAsReactor<T>
where
    S: ?Sized + ToOwned,
    T: Sink<S::Owned> + Unpin,
{
    type Error = T::Error;

    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
        block_on(self.send(state.to_owned()))
    }
}

impl<O, T> Sink<O> for SinkAsReactor<T>
where
    T: Sink<O>,
{
    type Error = T::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, state: O) -> Result<(), Self::Error> {
        self.project().sink.start_send(state)
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
    S: ToOwned + ?Sized,
{
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
    /// let mut reactor = Reactor::<str, Error = _>::from_sink(tx);
    ///
    /// thread::spawn(move || {
    ///     reactor.react("1");
    ///     reactor.react("1");
    ///     reactor.react("3");
    ///     reactor.react("5");
    ///     reactor.react("8");
    /// });
    ///
    /// assert_eq!(block_on_stream(rx).collect::<String>(), "11358".to_string());
    /// ```
    pub fn from_sink<T>(sink: T) -> SinkAsReactor<T>
    where
        T: Sink<S::Owned, Error = E> + Unpin,
    {
        SinkAsReactor { sink }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use proptest::prelude::*;
    use std::{string::String, vec::Vec};

    #[cfg_attr(tarpaulin, skip)]
    impl<T: Unpin, E: Unpin> Sink<T> for MockReactor<T, E> {
        type Error = E;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, state: T) -> Result<(), Self::Error> {
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
        fn deref(mut sink: Vec<String>) {
            let mut reactor = Reactor::<str, Error = _>::from_sink(sink.clone());

            assert_eq!(reactor.deref(), &sink);
            assert_eq!(reactor.deref_mut(), &mut sink);
        }

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
            assert_eq!(block_on(reactor.send(state)), result);
            assert_eq!(block_on(reactor.close()), Ok(()));
        }
    }
}
