use crate::reactor::*;
use derive_deref::{Deref, DerefMut};
use futures::executor::block_on;
use futures::sink::{Sink, SinkExt};
use futures::task::{Context, Poll};
use pin_project::*;
use std::{ops::DerefMut, pin::Pin};

#[pin_project]
#[derive(Deref, DerefMut)]
struct SinkAsReactor<T> {
    #[pin]
    sink: T,
}

impl<S, T> Reactor<S> for SinkAsReactor<T>
where
    S: Clone,
    T: Sink<S> + Unpin,
{
    type Error = T::Error;

    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
        block_on(self.send(state.clone()))
    }
}

impl<S, T> Sink<S> for SinkAsReactor<T>
where
    T: Sink<S>,
{
    type Error = T::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, state: S) -> Result<(), Self::Error> {
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
    S: Clone,
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
    /// let mut reactor = Reactor::<Error = _>::from_sink(tx);
    ///
    /// thread::spawn(move || {
    ///     reactor.react(&1);
    ///     reactor.react(&1);
    ///     reactor.react(&3);
    ///     reactor.react(&5);
    ///     reactor.react(&8);
    /// });
    ///
    /// assert_eq!(block_on_stream(rx).collect::<Vec<_>>(), vec![1, 1, 3, 5, 8]);
    /// ```
    #[cfg(feature = "async")]
    pub fn from_sink<T>(
        sink: T,
    ) -> impl Reactor<S, Error = E> + Sink<S, Error = E> + DerefMut<Target = T>
    where
        T: Sink<S, Error = E> + Unpin,
    {
        SinkAsReactor { sink }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use proptest::prelude::*;

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
        fn react(state: u8, result: Result<(), u8>) {
            let mut mock = MockReactor::new();

            mock.expect_react()
                .with(eq(state))
                .times(1)
                .return_const(result);

            let mut reactor = Reactor::<Error = _>::from_sink(mock);
            assert_eq!(Reactor::react(&mut reactor, &state), result);
        }

        #[test]
        fn sink(state: u8, result: Result<(), u8>) {
            let mut mock = MockReactor::new();

            mock.expect_react()
                .with(eq(state))
                .times(1)
                .return_const(result);

            let mut reactor = Reactor::<Error = _>::from_sink(mock);
            assert_eq!(block_on(reactor.send(state)), result);
            assert_eq!(block_on(reactor.close()), Ok(()));
        }
    }
}
