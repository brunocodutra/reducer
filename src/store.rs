use reactor::Reactor;
use reducer::Reducer;
use std::mem;

/// A reactive state container that manages the state of your application.
///
/// Store promotes reactive programming by encapsulating the state of your application and
/// notifying a [reactor](struct.Store.html#method.subscribe) upon every change.
///
/// The only way to mutate the internal state managed by Store is by
/// [dispatching](struct.Store.html#method.dispatch) actions on it.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Store<R: Reducer, S: Reactor<R>> {
    state: R,
    reactor: S,
}

impl<R: Reducer, S: Reactor<R>> Store<R, S> {
    /// Constructs the store given the initial state and a reactor.
    pub fn new(state: R, reactor: S) -> Self {
        Self { state, reactor }
    }

    /// Updates the state via [`<R as Reducer>::reduce`](trait.Reducer.html#tymethod.reduce) and
    /// notifies the reactor, returning the result of calling
    /// [`<S as Reactor<R>>::react`](trait.Reactor.html#tymethod.react) with a reference to the
    /// new state.
    pub fn dispatch(&mut self, action: impl Into<R::Action>) -> Result<(), S::Error> {
        self.state.reduce(action.into());
        self.reactor.react(&self.state)
    }

    /// Replaces the reactor and returns the previous one.
    pub fn subscribe(&mut self, reactor: impl Into<S>) -> S {
        mem::replace(&mut self.reactor, reactor.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reactor::MockReactor;
    use reducer::MockReducer;

    #[test]
    fn default() {
        let store = Store::<MockReducer<()>, MockReactor<_>>::default();

        assert_eq!(store.state, MockReducer::default());
        assert_eq!(store.reactor, MockReactor::default());
    }

    #[test]
    fn new() {
        let state = MockReducer::new(vec![42]);
        let reactor = MockReactor::new(vec![state.clone()]);
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

        assert!(store.dispatch(5).is_ok());
        assert!(store.dispatch(1).is_ok());
        assert!(store.dispatch(3).is_ok());

        store.reactor.set_result(Err);
        assert!(store.dispatch(false).is_err());
        store.reactor.set_result(Ok);

        assert_eq!(store.state, MockReducer::new(vec![5, 1, 3, false.into()]));

        assert_eq!(
            store.reactor,
            MockReactor::new(vec![
                MockReducer::new(vec![5]),
                MockReducer::new(vec![5, 1]),
                MockReducer::new(vec![5, 1, 3]),
            ]),
        );
    }

    #[test]
    fn subscribe() {
        let mut store = Store::new(MockReducer::default(), None);

        assert!(store.dispatch(0).is_ok());

        store.subscribe(Some(MockReactor::default()));

        assert!(store.dispatch(5).is_ok());
        assert!(store.dispatch(1).is_ok());
        assert!(store.dispatch(3).is_ok());

        assert_eq!(store.state, MockReducer::new(vec![0, 5, 1, 3]));

        assert_eq!(
            store.reactor,
            Some(MockReactor::new(vec![
                MockReducer::new(vec![0, 5]),
                MockReducer::new(vec![0, 5, 1]),
                MockReducer::new(vec![0, 5, 1, 3]),
            ])),
        );
    }
}
