use crate::dispatcher::Dispatcher;
use crate::reactor::Reactor;
use crate::reducer::Reducer;
use std::{mem, ops::Deref};

/// A reactive state container.
///
/// The only way to mutate the internal state managed by [`Store`] is by
/// [dispatching] actions on it.
/// The associated [`Reactor`] is notified upon every state transition.
///
/// [dispatching]: trait.Dispatcher.html#tymethod.dispatch
///
/// ## Example
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
///     type Output = io::Result<()>;
///     fn react(&self, state: &Calculator) -> Self::Output {
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
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Store<S, R: Reactor<S>> {
    state: S,
    reactor: R,
}

impl<S, R: Reactor<S>> Store<S, R> {
    /// Constructs the Store given the initial state and a [`Reactor`].
    pub fn new(state: S, reactor: R) -> Self {
        Self { state, reactor }
    }

    /// Replaces the [`Reactor`] and returns the previous one.
    pub fn subscribe(&mut self, reactor: impl Into<R>) -> R {
        mem::replace(&mut self.reactor, reactor.into())
    }
}

/// View Store as a read-only owning smart pointer to the state.
impl<S, R: Reactor<S>> Deref for Store<S, R> {
    type Target = S;

    /// Grants read access to the current state.
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<A, S, R> Dispatcher<A> for Store<S, R>
where
    S: Reducer<A>,
    R: Reactor<S>,
{
    type Output = R::Output;

    /// Updates the state via [`Reducer::reduce`] and notifies the [`Reactor`],
    /// returning the result of calling [`Reactor::react`] with a reference
    /// to the new state.
    fn dispatch(&mut self, action: A) -> R::Output {
        self.state.reduce(action);
        self.reactor.react(&self.state)
    }
}

#[cfg(feature = "async")]
use futures::sink::Sink;

#[cfg(feature = "async")]
use futures::task::{Context, Poll};

#[cfg(feature = "async")]
use std::pin::Pin;

#[cfg(feature = "async")]
use pin_utils::unsafe_pinned;

#[cfg(feature = "async")]
impl<S, R: Reactor<S>> Store<S, R> {
    unsafe_pinned!(state: S);
    unsafe_pinned!(reactor: R);
}

impl<S: Unpin, R: Reactor<S> + Unpin> Unpin for Store<S, R> {}

/// View Store as a Sink of actions (requires [`async`]).
///
/// [`async`]: index.html#experimental-features
#[cfg(feature = "async")]
impl<A, S, R, E> Sink<A> for Store<S, R>
where
    S: Reducer<A> + Unpin + Clone,
    R: Reactor<S, Output = Result<(), E>> + Sink<S, SinkError = E>,
{
    type SinkError = E;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        self.reactor().poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, action: A) -> Result<(), Self::SinkError> {
        let state = {
            let state: &mut S = self.as_mut().state().get_mut();
            state.reduce(action);
            state.clone()
        };

        self.reactor().start_send(state)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        self.reactor().poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        self.reactor().poll_close(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

    #[cfg(feature = "async")]
    use futures::{executor::block_on, sink::SinkExt};

    #[test]
    fn default() {
        let store = Store::<MockReducer<()>, MockReactor<_>>::default();

        assert_eq!(store.state, MockReducer::default());
        assert_eq!(store.reactor, MockReactor::default());
    }

    proptest! {
        #[test]
        fn new(actions: Vec<char>) {
            let state = MockReducer::new(actions);
            let reactor = MockReactor::default();
            let store = Store::new(state.clone(), &reactor);

            assert_eq!(store.state, state);
            assert_eq!(store.reactor, &reactor);
        }
    }

    proptest! {
        #[test]
        fn clone(actions: Vec<char>) {
            let store = Store::new(MockReducer::new(actions), MockReactor::default());
            assert_eq!(store, store.clone());
        }
    }

    proptest! {
        #[test]
        fn deref(actions: Vec<char>) {
            let store = Store::new(MockReducer::new(actions), MockReactor::default());
            assert_eq!(*store, store.state);
        }
    }

    proptest! {
        #[test]
        fn subscribe(actions: Vec<char>) {
            let state = MockReducer::new(actions);
            let mut store = Store::new(state.clone(), Some(MockReactor::default()));

            assert_eq!(store.state, state);
            assert_eq!(store.reactor, Some(MockReactor::default()));

            assert_eq!(store.subscribe(None), Some(MockReactor::default()));

            assert_eq!(store.state, state);
            assert_eq!(store.reactor, None);

            assert_eq!(store.subscribe(MockReactor::default()), None);

            assert_eq!(store.state, state);
            assert_eq!(store.reactor, Some(MockReactor::default()));
        }
    }

    proptest! {
        #[test]
        fn dispatch(actions: Vec<char>) {
            let mut store = Store::<MockReducer<_>, MockReactor<_>>::default();

            for (i, &action) in actions.iter().enumerate() {
                assert_eq!(store.dispatch(action), Ok(()));

                // The state is updated.
                assert_eq!(store.state, MockReducer::new(&actions[0..=i]));

                // The reactor is notified.
                assert_eq!(
                    store.reactor,
                    MockReactor::new(
                        (0..=i)
                            .map(|j| MockReducer::new(&actions[0..=j]))
                            .collect::<Vec<_>>(),
                    )
                );
            }
        }
    }

    proptest! {
        #[cfg(feature = "async")]
        #[test]
        fn sink(actions: Vec<char>) {
            let mut store = Store::<MockReducer<_>, MockReactor<_>>::default();

            for (i, &action) in actions.iter().enumerate() {
                // Futures do nothing unless polled, so the action is effectivelly dropped.
                drop(store.send(action));

                // No change.
                assert_eq!(store.state, MockReducer::new(&actions[0..i]));

                // No change.
                assert_eq!(
                    store.reactor,
                    MockReactor::new(
                        (0..i)
                            .map(|j| MockReducer::new(&actions[0..=j]))
                            .collect::<Vec<_>>(),
                    )
                );

                // Polling the future to completion guarantees the action is delivered.
                assert_eq!(block_on(store.send(action)), Ok(()));

                // The state is updated.
                assert_eq!(store.state, MockReducer::new(&actions[0..=i]));

                // The reactor is notified.
                assert_eq!(
                    store.reactor,
                    MockReactor::new(
                        (0..=i)
                            .map(|j| MockReducer::new(&actions[0..=j]))
                            .collect::<Vec<_>>(),
                    )
                );
            }
        }
    }
}
