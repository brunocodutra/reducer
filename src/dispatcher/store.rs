use crate::dispatcher::Dispatcher;
use crate::reactor::Reactor;
use crate::reducer::Reducer;
use std::mem;

/// A reactive state container.
///
/// The only way to mutate the internal state managed by Store is by
/// [dispatching](trait.Dispatcher.html) actions on it.
/// The associated reactor is notified upon every state transition.
///
/// ## Example
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

    /// Updates the state via [`<R as Reducer<A>>::reduce`](trait.Reducer.html#tymethod.reduce) and
    /// notifies the reactor, returning the result of calling
    /// [`<S as Reactor<R>>::react`](trait.Reactor.html#tymethod.react) with a reference to the
    /// new state.
    fn dispatch(&mut self, action: A) -> S::Output {
        self.state.reduce(action);
        self.reactor.react(&self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;

    #[test]
    fn default() {
        let store = Store::<MockReducer<()>, MockReactor<_>>::default();

        assert_eq!(store.state, MockReducer::default());
        assert_eq!(store.reactor, MockReactor::default());
    }

    #[test]
    fn new() {
        let state = MockReducer::new(vec![42]);
        let reactor = MockReactor::default();
        let store = Store::new(state.clone(), &reactor);

        assert_eq!(store.state, state);
        assert_eq!(store.reactor, &reactor);
    }

    #[test]
    fn clone() {
        let store = Store::new(MockReducer::<()>::default(), MockReactor::default());
        assert_eq!(store, store.clone());
    }

    #[test]
    fn dispatch() {
        let mut store = Store::<MockReducer<_>, MockReactor<_>>::default();

        assert_eq!(store.dispatch(5), MockReducer::new(vec![5]));
        assert_eq!(store.dispatch(1), MockReducer::new(vec![5, 1]));
        assert_eq!(store.dispatch(3), MockReducer::new(vec![5, 1, 3]));
    }

    #[test]
    fn subscribe() {
        let mut store: Store<_, Option<MockReactor<_>>> = Store::new(MockReducer::default(), None);

        assert_eq!(store.dispatch(0), None);

        store.subscribe(Some(MockReactor::default()));

        assert_eq!(store.dispatch(5), Some(MockReducer::new(vec![0, 5])));
        assert_eq!(store.dispatch(1), Some(MockReducer::new(vec![0, 5, 1])));
        assert_eq!(store.dispatch(3), Some(MockReducer::new(vec![0, 5, 1, 3])));
    }
}
