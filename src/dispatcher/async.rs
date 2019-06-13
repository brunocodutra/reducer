use crate::dispatcher::Dispatcher;
use futures::channel::mpsc;
use futures::executor::block_on;
use futures::future::{FutureExt, FutureObj, RemoteHandle};
use futures::sink::{Sink, SinkExt};
use futures::stream::StreamExt;
use futures::task::{Context, Poll, Spawn, SpawnError};
use std::{error::Error, fmt, pin::Pin};

/// Trait for types that can spawn [`Dispatcher`]s as an asynchronous task (requires [`async`]).
///
/// [`async`]: index.html#experimental-features
pub trait SpawnDispatcher {
    /// Spawns a [`Dispatcher`] as a task that will listen to actions dispatched through the
    /// [`AsyncDispatcher`] returned.
    ///
    /// The task completes
    /// * successfuly if [`AsyncDispatcher`] (or the last of its clones) is dropped.
    /// * successfuly if [`RemoteHandle`] is is dropped, unless [`RemoteHandle::forget`] is called.
    /// * with an error if [`Dispatcher::dispatch`] fails.
    ///     * The error can be retrieved by polling [`RemoteHandle`] to completion.
    #[allow(clippy::type_complexity)]
    fn spawn_dispatcher<D, A, E>(
        &mut self,
        dispatcher: D,
    ) -> Result<(AsyncDispatcher<A, E>, RemoteHandle<D::Output>), SpawnError>
    where
        D: Dispatcher<A, Output = Result<(), E>> + Sink<A, SinkError = E> + Send + 'static,
        A: Send + 'static,
        E: Send + 'static;
}

impl<S> SpawnDispatcher for S
where
    S: Spawn + ?Sized,
{
    #[allow(clippy::type_complexity)]
    fn spawn_dispatcher<D, A, E>(
        &mut self,
        dispatcher: D,
    ) -> Result<(AsyncDispatcher<A, E>, RemoteHandle<D::Output>), SpawnError>
    where
        D: Dispatcher<A, Output = Result<(), E>> + Sink<A, SinkError = E> + Send + 'static,
        A: Send + 'static,
        E: Send + 'static,
    {
        let (tx, rx) = mpsc::channel(0);
        let (future, handle) = rx.forward(dispatcher).remote_handle();
        self.spawn_obj(FutureObj::new(Box::new(future)))?;
        Ok((AsyncDispatcher(tx), handle))
    }
}

/// A handle that allows dispatching actions on a [spawned] [`Dispatcher`] (requires [`async`]).
///
/// [`AsyncDispatcher`] requires all actions to be of the same type `A`.
/// An effective way to fulfill this requirement is to define actions as `enum` variants.
///
/// This type is a just lightweight handle that may be cloned and sent to other threads.
///
/// [spawned]: trait.SpawnDispatcher.html
/// [`async`]: index.html#experimental-features
///
/// ## Example
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
///     type Output = io::Result<()>;
///     fn react(&self, state: &Calculator) -> Self::Output {
///         io::stdout().write_fmt(format_args!("{}\n", state.0))
///     }
/// }
///
/// // Implementing Sink for Console, means it can asynchronously react to state changes.
/// impl Sink<Calculator> for Console {
///     type SinkError = io::Error;
///
///     fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
///         Poll::Ready(Ok(()))
///     }
///
///     fn start_send(self: Pin<&mut Self>, state: Calculator) -> io::Result<()> {
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
///     let (mut dispatcher, handle) = executor.spawn_dispatcher(store).unwrap();
///
///     dispatcher.dispatch(Action::Add(5))?; // eventually displays "5"
///     dispatcher.dispatch(Action::Mul(3))?; // eventually displays "15"
///     dispatcher.dispatch(Action::Sub(1))?; // eventually displays "14"
///     dispatcher.dispatch(Action::Div(7))?; // eventually displays "2"
///
///     // Dropping the AsyncDispatcher signals to the background task that
///     // it can terminate once all pending actions have been processed.
///     drop(dispatcher);
///
///     // Wait for the background task to terminate.
///     block_on(handle)?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AsyncDispatcher<A, E>(mpsc::Sender<Result<A, E>>);

/// The error returned when [`AsyncDispatcher`] is unable to dispatch an action (requires [`async`]).
///
/// [`async`]: index.html#experimental-features
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AsyncDispatcherError {
    /// The [spawned] [`Dispatcher`] has terminated and cannot receive further actions.
    ///
    /// [spawned]: trait.SpawnDispatcher.html
    Terminated,
}

impl fmt::Display for AsyncDispatcherError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "The spawned Dispatcher has terminated and cannot receive further actions"
        )
    }
}

impl Error for AsyncDispatcherError {}

impl<A, E> Dispatcher<A> for AsyncDispatcher<A, E> {
    /// Either confirmation that action has been dispatched or the reason why not.
    type Output = Result<(), AsyncDispatcherError>;

    /// Sends an action to the associated [spawned] [`Dispatcher`].
    ///
    /// Once this call returns, the action may or may not have taken effect,
    /// but it's guaranteed to eventually do,
    /// unless the [spawned] [`Dispatcher`] terminates in between.
    ///
    /// [spawned]: trait.SpawnDispatcher.html
    fn dispatch(&mut self, action: A) -> Self::Output {
        block_on(self.send(action))
    }
}

impl<A, E> Sink<A> for AsyncDispatcher<A, E> {
    type SinkError = AsyncDispatcherError;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::SinkError>> {
        Pin::new(&mut self.0)
            .poll_ready(cx)
            .map_err(|_| AsyncDispatcherError::Terminated)
    }

    fn start_send(mut self: Pin<&mut Self>, action: A) -> Result<(), Self::SinkError> {
        Pin::new(&mut self.0)
            .start_send(Ok(action))
            .map_err(|_| AsyncDispatcherError::Terminated)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::SinkError>> {
        Pin::new(&mut self.0)
            .poll_flush(cx)
            .map_err(|_| AsyncDispatcherError::Terminated)
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::SinkError>> {
        Pin::new(&mut self.0)
            .poll_close(cx)
            .map_err(|_| AsyncDispatcherError::Terminated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use crate::Store;
    use futures::channel::mpsc::channel;
    use futures::executor::*;
    use futures::stream::*;
    use lazy_static::lazy_static;
    use proptest::*;
    use std::{error::Error, thread};

    lazy_static! {
        static ref POOL: ThreadPool = ThreadPool::new().unwrap();
    }

    proptest! {
        #[test]
        fn dispatch(actions: Vec<u8>) {
            let (tx, rx) = channel(actions.len());
            let store = Store::new(Mock::default(), tx);
            let mut executor = POOL.clone();
            let (mut dispatcher, handle) = executor.spawn_dispatcher(store).unwrap();

            for &action in &actions {
                assert_eq!(dispatcher.dispatch(action), Ok(()));
            }

            drop(dispatcher);

            assert_eq!(block_on(handle), Ok(()));

            assert_eq!(
                block_on_stream(rx).collect::<Vec<_>>(),
                (0..actions.len())
                    .map(|i| Mock::new(&actions[0..=i]))
                    .collect::<Vec<_>>()
            );
        }
    }

    proptest! {
        #[test]
        fn sink(actions: Vec<u8>) {
            let (tx, rx) = channel(actions.len());
            let store = Store::new(Mock::default(), tx);
            let mut executor = POOL.clone();
            let (mut dispatcher, handle) = executor.spawn_dispatcher(store).unwrap();

            assert_eq!(
                block_on(dispatcher.send_all(&mut iter(actions.clone()))),
                Ok(())
            );

            drop(dispatcher);

            assert_eq!(block_on(handle), Ok(()));

            assert_eq!(
                block_on_stream(rx).collect::<Vec<_>>(),
                (0..actions.len())
                    .map(|i| Mock::new(&actions[0..=i]))
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn error() -> Result<(), Box<dyn Error>> {
        let (tx, rx) = channel(0);
        let store = Store::new(Mock::default(), tx);
        let mut executor = POOL.clone();
        let (mut dispatcher, handle) = executor.spawn_dispatcher(store).unwrap();

        drop(rx);

        // Poll the spawned dispatcher so it sees the dead channel.
        dispatcher.dispatch(()).ok();

        assert_ne!(block_on(handle), Ok(()));

        while let Ok(()) = dispatcher.dispatch(()) {
            // Wait for the information to propagate,
            // that the spawned dispatcher has terminated.
            thread::yield_now();
        }

        assert_eq!(
            dispatcher.dispatch(()),
            Err(AsyncDispatcherError::Terminated)
        );

        Ok(())
    }
}
