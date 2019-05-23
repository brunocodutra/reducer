use crate::dispatcher::Dispatcher;
use crate::reactor::Reactor;
use crate::reducer::Reducer;
use std::mem;

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
pub struct Store<R, S: Reactor<R>> {
    state: R,
    reactor: S,
}

impl<R, S: Reactor<R>> Store<R, S> {
    /// Constructs the Store given the initial state and a reactor.
    pub fn new(state: R, reactor: S) -> Self {
        Self { state, reactor }
    }

    /// Replaces the reactor and returns the previous one.
    pub fn subscribe(&mut self, reactor: impl Into<S>) -> S {
        mem::replace(&mut self.reactor, reactor.into())
    }
}

impl<A, R, S> Dispatcher<A> for Store<R, S>
where
    R: Reducer<A>,
    S: Reactor<R>,
{
    type Output = S::Output;

    /// Updates the state via [`Reducer<A>::reduce`][reduce] and notifies the reactor,
    /// returning the result of calling [`Reactor<R>::react`][react] with a reference
    /// to the new state.
    ///
    /// [reduce]: trait.Reducer.html#tymethod.reduce
    /// [react]: trait.Reactor.html#tymethod.react
    fn dispatch(&mut self, action: A) -> S::Output {
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
        fn new(actions: Vec<u8>) {
            let state = MockReducer::new(actions);
            let reactor = MockReactor::default();
            let store = Store::new(state.clone(), &reactor);

            assert_eq!(store.state, state);
            assert_eq!(store.reactor, &reactor);
        }
    }

    proptest! {
        #[test]
        fn clone(actions: Vec<u8>) {
            let store = Store::new(MockReducer::new(actions), MockReactor::default());
            assert_eq!(store, store.clone());
        }
    }

    proptest! {
        #[test]
        fn subscribe(actions: Vec<u8>) {
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
        fn dispatch(actions: Vec<u8>) {
            let mut store = Store::<MockReducer<_>, MockReactor<_>>::default();

            for (i, &action) in actions.iter().enumerate() {
                assert_eq!(
                    store.dispatch(action),
                    MockReducer::new(actions[0..=i].into())
                );
            }
        }
    }
}
