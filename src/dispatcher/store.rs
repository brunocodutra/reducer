use crate::dispatcher::Dispatcher;
use crate::reactor::Reactor;
use crate::reducer::Reducer;
use core::{mem, ops::Deref};

#[cfg(feature = "async")]
use pin_project::*;

/// A reactive state container.
///
/// The only way to mutate the internal state managed by [`Store`] is by
/// [dispatching] actions on it.
/// The associated [`Reactor`] is notified upon every state transition.
///
/// [dispatching]: trait.Dispatcher.html#tymethod.dispatch
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
#[cfg_attr(feature = "async", pin_project)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Store<S, R: Reactor<S>> {
    state: S,
    #[cfg_attr(feature = "async", pin)]
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
    use futures::sink::Sink;
    use futures::task::{Context, Poll};
    use std::{borrow::ToOwned, pin::Pin};

    /// View Store as a Sink of actions (requires [`async`]).
    ///
    /// [`async`]: index.html#optional-features
    impl<A, S, R, E> Sink<A> for Store<S, R>
    where
        S: Reducer<A> + ToOwned,
        R: Reactor<S, Error = E> + Sink<S::Owned, Error = E>,
    {
        type Error = E;

        fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.project().reactor.poll_ready(cx)
        }

        #[project]
        fn start_send(self: Pin<&mut Self>, action: A) -> Result<(), Self::Error> {
            #[project]
            let Store { state, reactor } = self.project();
            state.reduce(action);
            reactor.start_send(state.to_owned())
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.project().reactor.poll_flush(cx)
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.project().reactor.poll_close(cx)
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
    use futures::{executor::block_on, sink::SinkExt};

    #[test]
    fn default() {
        Store::<MockReducer<()>, MockReactor<_, ()>>::default();
    }

    #[test]
    fn deref() {
        let store = Store::new(MockReducer::<()>::new(), MockReactor::<_, ()>::new());
        assert_eq!(&*store as *const _, &store.state as *const _);
    }

    proptest! {
        #[test]
        fn new(a: usize, b: usize) {
            let mut reducer = MockReducer::<()>::new();
            reducer.expect_id().return_const(a);

            let mut reactor = MockReactor::<_, ()>::new();
            reactor.expect_id().return_const(b);

            let store = Store::new(reducer, reactor);

            assert_eq!(store.state.id(), a);
            assert_eq!(store.reactor.id(), b);
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

            let mut reactor = MockReactor::<_, ()>::new();
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
            let mut mock = MockReactor::<_, ()>::new();
            mock.expect_id().return_const(a);

            let mut store = Store::new(MockReducer::<()>::new(), mock);

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

            let mut store = Store::new(reducer, reactor);
            assert_eq!(block_on(store.send(action)), result);
            assert_eq!(block_on(store.close()), Ok(()));
        }
    }
}
