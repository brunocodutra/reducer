use crate::dispatcher::*;
use derive_more::{Deref, DerefMut};
use futures::executor::block_on;
use futures::sink::{Sink, SinkExt};
use futures::task::{Context, Poll};
use pin_project::*;
use std::pin::Pin;

/// A handle that allows dispatching actions on a [spawned] [`Dispatcher`] (requires [`async`]).
///
/// This type is a just lightweight handle that may be cloned and sent to other threads.
///
/// [spawned]: Store::into_task
/// [`async`]: index.html#optional-features
#[pin_project]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deref, DerefMut)]
pub struct AsyncDispatcher<T> {
    #[pin]
    sink: T,
}

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
        self.project().sink.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, action: A) -> Result<(), Self::Error> {
        self.project().sink.start_send(action)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_close(cx)
    }
}

impl<A, E> dyn Dispatcher<A, Output = Result<(), E>> {
    /// Adapts any type that implements [`Sink`] as a [`Dispatcher`] (requires [`async`]).
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
    /// let mut dispatcher = Dispatcher::<_, Output = _>::from_sink(tx);
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
    /// assert_eq!(block_on_stream(rx).collect::<Vec<u8>>(), vec![1, 1, 2, 3, 5, 8]);
    /// ```
    pub fn from_sink<T>(sink: T) -> AsyncDispatcher<T>
    where
        T: Sink<A, Error = E> + Unpin,
    {
        AsyncDispatcher { sink }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use proptest::prelude::*;
    use std::{ops::*, vec::Vec};

    proptest! {
        #[test]
        fn deref(mut sink: Vec<u8>) {
            let mut dispatcher = Dispatcher::<_, Output = _>::from_sink(sink.clone());

            assert_eq!(dispatcher.deref(), &sink);
            assert_eq!(dispatcher.deref_mut(), &mut sink);
        }

        #[test]
        fn dispatch(action: u8, result: Result<(), u8>) {
            let mut mock = MockDispatcher::new();

            mock.expect_dispatch()
                .with(eq(action))
                .times(1)
                .return_const(result);

            let mut dispatcher = Dispatcher::<_, Output = _>::from_sink(mock);
            assert_eq!(Dispatcher::dispatch(&mut dispatcher, action), result);
        }

        #[test]
        fn sink(action: u8, result: Result<(), u8>) {
            let mut mock = MockDispatcher::new();

            mock.expect_dispatch()
                .with(eq(action))
                .times(1)
                .return_const(result);

            let mut dispatcher = Dispatcher::<_, Output = _>::from_sink(mock);
            assert_eq!(block_on(dispatcher.send(action)), result);
            assert_eq!(block_on(dispatcher.close()), Ok(()));
        }
    }
}
