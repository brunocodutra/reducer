use crate::reactor::*;
use derive_more::{Deref, DerefMut, From};
use futures::sink::{Sink, SinkExt};
use pin_project::pin_project;
use std::task::{Context, Poll};
use std::{borrow::ToOwned, pin::Pin};

/// An adapter for [`Sink`]s that behaves as an asynchronous [`Reactor`] (requires [`async`]).
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
/// let mut reactor = AsyncReactor(tx);
///
/// thread::spawn(move || {
///     reactor.react(&'1');
///     reactor.react(&'1');
///     reactor.react(&'2');
///     reactor.react(&'3');
///     reactor.react(&'5');
///     reactor.react(&'8');
/// });
///
/// assert_eq!(block_on_stream(rx).collect::<String>(), "112358");
/// ```
#[pin_project]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, From, Deref, DerefMut)]
pub struct AsyncReactor<T>(#[pin] pub T);

impl<S, T, E> Reactor<S> for AsyncReactor<T>
where
    S: ?Sized,
    Self: for<'s> Sink<&'s S, Error = E> + Unpin,
{
    /// The reason why the state couldn't be sent through the sink.
    type Error = E;

    /// Sends an owned version of the state through the sink.
    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
        futures::executor::block_on(self.send(state))
    }
}

impl<S, T, O> Sink<&S> for AsyncReactor<T>
where
    S: ToOwned<Owned = O> + ?Sized,
    T: Sink<O>,
{
    type Error = T::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().0.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, state: &S) -> Result<(), Self::Error> {
        self.project().0.start_send(state.to_owned())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().0.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().0.poll_close(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use std::{ops::*, string::String, vec::Vec};
    use test_strategy::proptest;
    use tokio::runtime;

    #[proptest]
    fn deref(sink: Vec<u8>) {
        let mut dispatcher = AsyncReactor(sink.clone());

        assert_eq!(dispatcher.deref(), &sink);
        assert_eq!(dispatcher.deref_mut(), &sink);
    }

    #[proptest]
    fn react(state: String, result: Result<(), u8>) {
        let mut mock = MockReactor::new();

        mock.expect_react()
            .with(eq(state.clone()))
            .once()
            .return_const(result);

        let mut reactor = AsyncReactor(mock);
        assert_eq!(Reactor::react(&mut reactor, state.as_str()), result);
    }

    #[proptest]
    fn sink(state: String, result: Result<(), u8>) {
        let rt = runtime::Builder::new_multi_thread().build()?;
        let mut mock = MockReactor::new();

        mock.expect_react()
            .with(eq(state.clone()))
            .once()
            .return_const(result);

        let mut reactor = AsyncReactor(mock);
        assert_eq!(rt.block_on(reactor.send(state.as_str())), result);
    }
}
