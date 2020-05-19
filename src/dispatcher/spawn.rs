use crate::dispatcher::*;
use futures::channel::mpsc::{channel, SendError, Sender};
use futures::future::{FutureExt, RemoteHandle, TryFuture};
use futures::sink::{Sink, SinkExt, SinkMapErr};
use futures::stream::StreamExt;
use futures::task::{Spawn, SpawnError, SpawnExt};
use thiserror::Error;

/// Trait for types that can spawn [`Dispatcher`]s as an asynchronous task (requires [`async`]).
///
/// [`async`]: index.html#optional-features
pub trait SpawnDispatcher<A, O, E> {
    /// The type of the result handle returned by [`spawn_dispatcher`].
    ///
    /// [`spawn_dispatcher`]: trait.SpawnDispatcher.html#tymethod.spawn_dispatcher
    type Handle: TryFuture<Ok = O, Error = E>;

    /// The type of the [`Dispatcher`] returned by [`spawn_dispatcher`].
    ///
    /// [`spawn_dispatcher`]: trait.SpawnDispatcher.html#tymethod.spawn_dispatcher
    type Dispatcher: Dispatcher<A>;

    /// Spawns a [`Dispatcher`] as a task that will listen to actions dispatched through the
    /// [`AsyncDispatcher`] returned.
    ///
    /// The task completes
    /// * successfully if [`AsyncDispatcher`] (or the last of its clones) is dropped or closed.
    /// * successfully if [`RemoteHandle`] is is dropped, unless [`RemoteHandle::forget`] is called.
    /// * with an error if [`Dispatcher::dispatch`] fails.
    ///     * The error can be retrieved by polling [`RemoteHandle`] to completion.
    ///
    /// Spawning a [`Dispatcher`] requires all actions to be of the same type `A`;
    /// an effective way of fulfilling this requirement is to define actions as `enum` variants.
    ///
    /// # Example
    ///
    /// ```rust
    /// use futures::executor::*;
    /// use futures::prelude::*;
    /// use futures::task::*;
    /// use reducer::*;
    /// use std::error::Error;
    /// use std::io::{self, Write};
    /// use std::pin::Pin;
    ///
    /// // The state of your app.
    /// #[derive(Clone)]
    /// struct Calculator(i32);
    ///
    /// // Actions the user can trigger.
    /// enum Action {
    ///     Add(i32),
    ///     Sub(i32),
    ///     Mul(i32),
    ///     Div(i32),
    /// }
    ///
    /// impl Reducer<Action> for Calculator {
    ///     fn reduce(&mut self, action: Action) {
    ///         match action {
    ///             Action::Add(x) => self.0 += x,
    ///             Action::Sub(x) => self.0 -= x,
    ///             Action::Mul(x) => self.0 *= x,
    ///             Action::Div(x) => self.0 /= x,
    ///         }
    ///     }
    /// }
    ///
    /// // The user interface.
    /// struct Console;
    ///
    /// impl Reactor<Calculator> for Console {
    ///     type Error = io::Error;
    ///     fn react(&mut self, state: &Calculator) -> io::Result<()> {
    ///         io::stdout().write_fmt(format_args!("{}\n", state.0))
    ///     }
    /// }
    ///
    /// // Implementing Sink for Console, means it can asynchronously react to state changes.
    /// impl Sink<Calculator> for Console {
    ///     type Error = io::Error;
    ///
    ///     fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
    ///         Poll::Ready(Ok(()))
    ///     }
    ///
    ///     fn start_send(mut self: Pin<&mut Self>, state: Calculator) -> io::Result<()> {
    ///         self.react(&state)
    ///     }
    ///
    ///     fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
    ///         Poll::Ready(Ok(()))
    ///     }
    ///
    ///     fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
    ///         Poll::Ready(Ok(()))
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let store = Store::new(Calculator(0), Console);
    ///
    ///     // Spin up a thread-pool.
    ///     let mut executor = ThreadPool::new()?;
    ///
    ///     // Process incoming actions on a background task.
    ///     let (mut dispatcher, handle) = executor.spawn_dispatcher(store)?;
    ///
    ///     dispatcher.dispatch(Action::Add(5))?; // eventually displays "5"
    ///     dispatcher.dispatch(Action::Mul(3))?; // eventually displays "15"
    ///     dispatcher.dispatch(Action::Sub(1))?; // eventually displays "14"
    ///     dispatcher.dispatch(Action::Div(7))?; // eventually displays "2"
    ///
    ///     // Closing the AsyncDispatcher signals to the background task that
    ///     // it can terminate once all pending actions have been processed.
    ///     block_on(dispatcher.close())?;
    ///
    ///     // Wait for the background task to terminate.
    ///     block_on(handle)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn spawn_dispatcher<D>(&mut self, d: D) -> Result<(Self::Dispatcher, Self::Handle), SpawnError>
    where
        D: Dispatcher<A, Output = Result<O, E>> + Sink<A, Error = E> + Send + 'static;
}

/// The error returned when [`AsyncDispatcher`] is unable to dispatch an action (requires [`async`]).
///
/// [`async`]: index.html#optional-features
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Error)]
pub enum AsyncDispatcherError {
    /// The [spawned] [`Dispatcher`] has terminated and cannot receive further actions.
    ///
    /// [spawned]: trait.SpawnDispatcher.html
    #[error("The spawned Dispatcher has terminated and cannot receive further actions")]
    Terminated,
}

impl<A, E, S> SpawnDispatcher<A, (), E> for S
where
    A: Send + 'static,
    E: Send + 'static,
    S: Spawn + ?Sized,
{
    type Handle = RemoteHandle<Result<(), E>>;

    #[doc(hidden)]
    #[allow(clippy::type_complexity)]
    type Dispatcher = AsyncDispatcher<SinkMapErr<Sender<A>, fn(SendError) -> AsyncDispatcherError>>;

    fn spawn_dispatcher<D>(&mut self, d: D) -> Result<(Self::Dispatcher, Self::Handle), SpawnError>
    where
        D: Dispatcher<A, Output = Result<(), E>> + Sink<A, Error = E> + Send + 'static,
    {
        let (tx, rx) = channel(0);
        let (future, handle) = rx.map(Ok).forward(d).remote_handle();
        let dispatcher: Self::Dispatcher = Dispatcher::<_, Output = _>::from_sink(
            tx.sink_map_err(|_| AsyncDispatcherError::Terminated),
        );

        self.spawn(future)?;
        Ok((dispatcher, handle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::*;
    use lazy_static::lazy_static;
    use mockall::predicate::*;
    use proptest::prelude::*;
    use std::thread::yield_now;

    lazy_static! {
        static ref POOL: ThreadPool = ThreadPool::new().unwrap();
    }

    proptest! {
        #[test]
        fn dispatch(action: u8, result: Result<(), u8>) {
            let mut store = MockDispatcher::new();

            store
                .expect_dispatch()
                .with(eq(action))
                .times(1)
                .return_const(result);

            let mut executor = POOL.clone();
            let (mut dispatcher, handle) = executor.spawn_dispatcher(store)?;

            assert_eq!(dispatcher.dispatch(action), Ok(()));
            assert_eq!(block_on(dispatcher.close()), Ok(()));
            assert_eq!(block_on(handle), result);
        }

        #[test]
        fn error(action: u8, error: u8) {
            let mut store = MockDispatcher::new();

            store
                .expect_dispatch()
                .with(eq(action))
                .times(1)
                .return_const(Err(error));

            let mut executor = POOL.clone();
            let (mut dispatcher, handle) = executor.spawn_dispatcher(store)?;

            assert_eq!(dispatcher.dispatch(action), Ok(()));

            loop {
                match dispatcher.dispatch(action) {
                    // Wait for the information to propagate,
                    // that the spawned dispatcher has terminated.
                    Ok(()) => yield_now(),
                    Err(e) => break assert_eq!(e, AsyncDispatcherError::Terminated),
                }
            }

            assert_eq!(block_on(handle), Err(error));
        }

        #[test]
        fn sink(action: u8, result: Result<(), u8>) {
            let mut store = MockDispatcher::new();

            store
                .expect_dispatch()
                .with(eq(action))
                .times(1)
                .return_const(result);

            let mut executor = POOL.clone();
            let (mut dispatcher, handle) = executor.spawn_dispatcher(store)?;

            assert_eq!(block_on(dispatcher.send(action)), Ok(()));
            assert_eq!(block_on(dispatcher.close()), Ok(()));
            assert_eq!(block_on(handle), result);
        }
    }
}
