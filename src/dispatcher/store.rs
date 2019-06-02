use crate::dispatcher::Dispatcher;
use crate::reactor::Reactor;
use crate::reducer::Reducer;
use std::{mem, ops::Deref};

/// A reactive state container.
///
/// The only way to mutate the internal state managed by Store is by
/// [dispatching] actions on it.
/// The associated reactor is notified upon every state transition.
///
/// [dispatching]: trait.Dispatcher.html
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
/// struct Display;
///
/// impl Reactor<Calculator> for Display {
///     type Output = io::Result<()>;
///     fn react(&self, state: &Calculator) -> Self::Output {
///         io::stdout().write_fmt(format_args!("{}\n", state.0))
///     }
/// }
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let mut store = Store::new(Calculator(0), Display);
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
    /// Constructs the Store given the initial state and a reactor.
    pub fn new(state: S, reactor: R) -> Self {
        Self { state, reactor }
    }

    /// Replaces the reactor and returns the previous one.
    pub fn subscribe(&mut self, reactor: impl Into<R>) -> R {
        mem::replace(&mut self.reactor, reactor.into())
    }
}

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

    /// Updates the state via [`Reducer<A>::reduce`][reduce] and notifies the reactor,
    /// returning the result of calling [`Reactor<S>::react`][react] with a reference
    /// to the new state.
    ///
    /// [reduce]: trait.Reducer.html#tymethod.reduce
    /// [react]: trait.Reactor.html#tymethod.react
    fn dispatch(&mut self, action: A) -> R::Output {
        self.state.reduce(action);
        self.reactor.react(&self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

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
                store.dispatch(action);

                assert_eq!(store.state, MockReducer::new(&actions[0..=i]));

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
