use crate::dispatcher::*;
use derive_more::{Deref, DerefMut, From};
use futures::executor::block_on;
use futures::sink::{Sink, SinkExt};
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

/// An adapter for [`Sink`]s that behaves as an asynchronous [`Dispatcher`] (requires [`async`]).
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
/// let mut dispatcher = AsyncDispatcher(tx);
///
/// thread::spawn(move || {
///     dispatcher.dispatch(1);
///     dispatcher.dispatch(1);
///     dispatcher.dispatch(2);
///     dispatcher.dispatch(3);
///     dispatcher.dispatch(5);
///     dispatcher.dispatch(8);
/// });
///
/// assert_eq!(block_on_stream(rx).collect::<Vec<u8>>(), [1, 1, 2, 3, 5, 8]);
/// ```
#[pin_project]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, From, Deref, DerefMut)]
pub struct AsyncDispatcher<T>(#[pin] pub T);

impl<A, T> Dispatcher<A> for AsyncDispatcher<T>
where
    T: Sink<A> + Unpin,
{
    /// Either confirmation that action has been dispatched through the sink or the reason why not.
    type Output = Result<(), T::Error>;

    /// Sends an action through the sink.
    ///
    /// Once this call returns, the action may or may not have taken effect,
    /// but it's guaranteed to eventually do,
    /// unless the sink is closed in between.
    fn dispatch(&mut self, action: A) -> Self::Output {
        block_on(self.send(action))
    }
}

impl<A, T> Sink<A> for AsyncDispatcher<T>
where
    T: Sink<A>,
{
    type Error = T::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().0.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, action: A) -> Result<(), Self::Error> {
        self.project().0.start_send(action)
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
    use std::{ops::*, vec::Vec};
    use test_strategy::proptest;

    #[proptest]
    fn deref(sink: Vec<u8>) {
        let mut dispatcher = AsyncDispatcher(sink.clone());

        assert_eq!(dispatcher.deref(), &sink);
        assert_eq!(dispatcher.deref_mut(), &sink);
    }

    #[proptest]
    fn dispatch(action: u8, result: Result<(), u8>) {
        let mut mock = MockDispatcher::new();

        mock.expect_dispatch()
            .with(eq(action))
            .once()
            .return_const(result);

        let mut dispatcher = AsyncDispatcher(mock);
        assert_eq!(Dispatcher::dispatch(&mut dispatcher, action), result);
    }

    #[proptest]
    fn sink(action: u8, result: Result<(), u8>) {
        let mut mock = MockDispatcher::new();

        mock.expect_dispatch()
            .with(eq(action))
            .once()
            .return_const(result);

        let mut dispatcher = AsyncDispatcher(mock);
        assert_eq!(block_on(dispatcher.send(action)), result);
        assert_eq!(block_on(dispatcher.close()), Ok(()));
    }
}
