use crate::dispatcher::*;
use crate::reactor::Reactor;
use crate::reducer::Reducer;
use core::mem::replace;
use derive_more::Deref;

#[cfg(feature = "async")]
use pin_project::*;

/// A reactive state container.
///
/// The only way to mutate the internal state managed by [`Store`] is by
/// [dispatching] actions on it.
/// The associated [`Reactor`] is notified upon every state transition.
///
/// [dispatching]: Store::dispatch
///
/// # Example
///
/// ```rust
/// use reducer::*;
/// use std::error::Error;
/// use std::io::{self, Write};
///
/// // The state of your app.
/// struct Calculator(i32);
///
/// // Actions the user can trigger.
/// struct Add(i32);
/// struct Sub(i32);
/// struct Mul(i32);
/// struct Div(i32);
///
/// impl Reducer<Add> for Calculator {
///     fn reduce(&mut self, Add(x): Add) {
///         self.0 += x;
///     }
/// }
///
/// impl Reducer<Sub> for Calculator {
///     fn reduce(&mut self, Sub(x): Sub) {
///         self.0 -= x;
///     }
/// }
///
/// impl Reducer<Mul> for Calculator {
///     fn reduce(&mut self, Mul(x): Mul) {
///         self.0 *= x;
///     }
/// }
///
/// impl Reducer<Div> for Calculator {
///     fn reduce(&mut self, Div(x): Div) {
///         self.0 /= x;
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
/// fn main() -> Result<(), Box<dyn Error>> {
///     let mut store = Store::new(Calculator(0), Console);
///
///     store.dispatch(Add(5))?; // displays "5"
///     store.dispatch(Mul(3))?; // displays "15"
///     store.dispatch(Sub(1))?; // displays "14"
///     store.dispatch(Div(7))?; // displays "2"
///
///     Ok(())
/// }
/// ```
#[cfg_attr(feature = "async", pin_project(project = StoreProjection))]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deref)]
pub struct Store<S, R> {
    #[deref]
    state: S,
    #[cfg_attr(feature = "async", pin)]
    reactor: R,
}

impl<S, R> Store<S, R> {
    /// Constructs the Store given the initial state and a [`Reactor`].
    pub fn new(state: S, reactor: R) -> Self {
        Self { state, reactor }
    }

    /// Replaces the [`Reactor`] and returns the previous one.
    pub fn subscribe(&mut self, reactor: impl Into<R>) -> R {
        replace(&mut self.reactor, reactor.into())
    }
}

impl<A, S, R> Dispatcher<A> for Store<S, R>
where
    S: Reducer<A>,
    R: Reactor<S>,
{
    type Output = Result<(), R::Error>;

    /// Updates the state via [`Reducer::reduce`] and notifies the [`Reactor`],
    /// returning the result of calling [`Reactor::react`] with a reference
    /// to the new state.
    fn dispatch(&mut self, action: A) -> Self::Output {
        self.state.reduce(action);
        self.reactor.react(&self.state)
    }
}

#[cfg(feature = "async")]
mod sink {
    use super::*;
    use derive_more::{Display, Error};
    use futures::channel::mpsc::channel;
    use futures::prelude::*;
    use futures::sink::Sink;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    /// View Store as a Sink of actions (requires [`async`]).
    ///
    /// [`async`]: index.html#optional-features
    impl<A, S, R, E> Sink<A> for Store<S, R>
    where
        S: Reducer<A>,
        R: for<'s> Sink<&'s S, Error = E>,
    {
        type Error = E;

        fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.project().reactor.poll_ready(cx)
        }

        fn start_send(self: Pin<&mut Self>, action: A) -> Result<(), Self::Error> {
            let StoreProjection { state, reactor } = self.project();
            state.reduce(action);
            reactor.start_send(state)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.project().reactor.poll_flush(cx)
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.project().reactor.poll_close(cx)
        }
    }

    /// The error returned when dispatching an action to a [spawned] [`Store`] fails
    /// (requires [`async`]).
    ///
    /// [spawned]: Store::into_task
    /// [`async`]: index.html#optional-features
    #[derive(Debug, Display, Copy, Clone, Eq, PartialEq, Hash, Error)]
    pub enum AsyncDispatcherError {
        /// The [spawned] task has terminated and cannot receive further actions.
        ///
        /// [spawned]: Store::into_task
        #[display(fmt = "The spawned task has terminated and cannot receive further actions")]
        Terminated,
    }

    impl<S, R> Store<S, R> {
        /// Turns the [`Store`] into a task that can be spawned onto an executor
        /// (requires [`async`]).
        ///
        /// Once spawned, the task will receive actions dispatched through the [`Dispatcher`]
        /// returned.
        ///
        /// The task completes
        /// * successfully if the asynchronous [`Dispatcher`] (or the last of its clones)
        ///   is dropped or [closed].
        /// * with an error if [`Store::dispatch`] fails.
        ///
        /// Turning a [`Store`] into an asynchronous task requires all actions to be of the same
        /// type `A`; an effective way of fulfilling this requirement is to define actions as
        /// `enum` variants.
        ///
        /// [`async`]: index.html#optional-features
        /// [closed]: futures::sink::SinkExt::close
        ///
        /// # Example
        ///
        /// ```rust
        /// use smol::{block_on, spawn};
        /// use futures::prelude::*;
        /// use reducer::*;
        /// use std::error::Error;
        /// use std::io::{self, Write};
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
        /// fn main() -> Result<(), Box<dyn Error>> {
        ///     let store = Store::new(
        ///         Calculator(0),
        ///         Reactor::<_, Error = _>::from_sink(sink::unfold((), |_, state: Calculator| {
        ///             future::ready(io::stdout().write_fmt(format_args!("{}\n", state.0)))
        ///         })),
        ///     );
        ///
        ///     // Process incoming actions on a background task.
        ///     let (task, mut dispatcher) = store.into_task();
        ///     let handle = spawn(task);
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
        pub fn into_task<A, E>(
            self,
        ) -> (
            impl Future<Output = Result<(), E>>,
            impl Dispatcher<A, Output = Result<(), AsyncDispatcherError>>
                + Sink<A, Error = AsyncDispatcherError>
                + Clone,
        )
        where
            Self: Sink<A, Error = E>,
        {
            let (tx, rx) = channel(0);
            let future = rx.map(Ok).forward(self);
            let dispatcher = Dispatcher::<_, Output = _>::from_sink(
                tx.sink_map_err(|_| AsyncDispatcherError::Terminated),
            );

            (future, dispatcher)
        }
    }
}

#[cfg(feature = "async")]
pub use sink::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactor::MockReactor;
    use crate::reducer::MockReducer;
    use mockall::predicate::*;
    use proptest::prelude::*;

    #[cfg(feature = "async")]
    use futures::SinkExt;

    #[cfg(feature = "async")]
    use smol::{block_on, spawn};

    #[cfg(feature = "async")]
    use std::thread::yield_now;

    #[test]
    fn default() {
        Store::<(), ()>::default();
    }

    proptest! {
        #[test]
        fn deref(state: usize) {
            let store = Store::new(state, ());
            assert_eq!(&*store as *const _, &store.state as *const _);
        }

        #[test]
        fn new(state: usize, reactor: usize) {
            let store = Store::new(state, reactor);
            assert_eq!(store.state, state);
            assert_eq!(store.reactor, reactor);
        }

        #[test]
        fn clone(a: usize, b: usize) {
            let mut reducer = MockReducer::<()>::new();
            reducer.expect_id().return_const(a);
            reducer.expect_clone().times(1).returning(move || {
                let mut mock = MockReducer::new();
                mock.expect_id().return_const(a);
                mock
            });

            let mut reactor = MockReactor::<(), ()>::new();
            reactor.expect_id().return_const(b);
            reactor.expect_clone().times(1).returning(move || {
                let mut mock = MockReactor::new();
                mock.expect_id().return_const(b);
                mock
            });

            #[allow(clippy::redundant_clone)]
            let store = Store::new(reducer, reactor).clone();

            assert_eq!(store.state.id(), a);
            assert_eq!(store.reactor.id(), b);
        }

        #[test]
        fn subscribe(a: usize, b: usize) {
            let mut mock = MockReactor::<(), ()>::new();
            mock.expect_id().return_const(a);

            let mut store = Store::new((), mock);

            let mut mock = MockReactor::<_, ()>::new();
            mock.expect_id().return_const(b);

            assert_eq!(store.subscribe(mock).id(), a);
            assert_eq!(store.reactor.id(), b);
        }

        #[test]
        fn dispatch(action: u8, result: Result<(), u8>, id: usize) {
            let mut reducer = MockReducer::new();
            reducer.expect_id().return_const(id);
            reducer.expect_clone().never();
            reducer
                .expect_reduce()
                .with(eq(action))
                .times(1)
                .return_const(());

            let mut reactor = MockReactor::new();
            reactor
                .expect_react()
                .with(function(move |x: &MockReducer<_>| x.id() == id))
                .times(1)
                .return_const(result);

            let mut store = Store::new(reducer, reactor);
            assert_eq!(Dispatcher::dispatch(&mut store, action), result);
        }

        #[cfg(feature = "async")]
        #[test]
        fn sink(action: u8, result: Result<(), u8>, id: usize) {
            let mut reducer = MockReducer::new();
            reducer.expect_id().return_const(id);
            reducer.expect_clone().returning(move || {
                let mut mock = MockReducer::new();
                mock.expect_id().return_const(id);
                mock.expect_reduce().never();
                mock.expect_clone().never();
                mock
            });

            reducer
                .expect_reduce()
                .with(eq(action))
                .times(1)
                .return_const(());

            let mut reactor = MockReactor::new();
            reactor
                .expect_react()
                .with(function(move |x: &MockReducer<_>| x.id() == id))
                .times(1)
                .return_const(result);

            let mut store = Store::new(reducer, Reactor::<_, Error = _>::from_sink(reactor));
            assert_eq!(block_on(store.send(action)), result);
            assert_eq!(block_on(store.close()), Ok(()));
        }

        #[cfg(feature = "async")]
        #[test]
        fn task(action: u8, result: Result<(), u8>, id: usize) {
            let mut reducer = MockReducer::new();
            reducer.expect_id().return_const(id);
            reducer.expect_clone().returning(move || {
                let mut mock = MockReducer::new();
                mock.expect_id().return_const(id);
                mock.expect_reduce().never();
                mock.expect_clone().never();
                mock
            });

            reducer
                .expect_reduce()
                .with(eq(action))
                .times(1)
                .return_const(());

            let mut reactor = MockReactor::new();
            reactor
                .expect_react()
                .with(function(move |x: &MockReducer<_>| x.id() == id))
                .times(1)
                .return_const(result);

            let store = Store::new(reducer, Reactor::<_, Error = _>::from_sink(reactor));
            let (task, mut dispatcher) = store.into_task();

            let handle = spawn(task);

            assert_eq!(dispatcher.dispatch(action), Ok(()));
            assert_eq!(block_on(dispatcher.close()), Ok(()));
            assert_eq!(block_on(handle), result);
        }

        #[cfg(feature = "async")]
        #[test]
        fn error(action: u8, error: u8, id: usize) {
            let mut reducer = MockReducer::new();
            reducer.expect_id().return_const(id);
            reducer.expect_clone().returning(move || {
                let mut mock = MockReducer::new();
                mock.expect_id().return_const(id);
                mock.expect_reduce().never();
                mock.expect_clone().never();
                mock
            });

            reducer
                .expect_reduce()
                .with(eq(action))
                .times(1)
                .return_const(());

            let mut reactor = MockReactor::new();
            reactor
                .expect_react()
                .with(function(move |x: &MockReducer<_>| x.id() == id))
                .times(1)
                .return_const(Err(error));

            let store = Store::new(reducer, Reactor::<_, Error = _>::from_sink(reactor));
            let (task, mut dispatcher) = store.into_task();

            let handle = spawn(task);

            assert_eq!(dispatcher.dispatch(action), Ok(()));

            loop {
                match dispatcher.dispatch(action) {
                    // Wait for the information to propagate,
                    // that the spawned task has terminated.
                    Ok(()) => yield_now(),
                    Err(e) => break assert_eq!(e, AsyncDispatcherError::Terminated),
                }
            }

            assert_eq!(block_on(handle), Err(error));
        }
    }
}
