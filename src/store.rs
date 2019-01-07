use reactor::Reactor;
use reducer::Reducer;
use std::mem;

/// A reactive state container that manages the state of your application.
///
/// Store promotes reactive programming by encapsulating the state of your application and
/// notifying a [reactor](struct.Store.html#method.subscribe) upon every change.
/// The only way to mutate the internal state managed by Store is by
/// [dispatching](struct.Store.html#method.dispatch) actions on it.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Store<R, S: Reactor<R>> {
    state: R,
    reactor: S,
}

impl<R, S: Reactor<R>> Store<R, S> {
    /// Constructs the store given the initial state and a reactor.
    pub fn new(state: R, reactor: S) -> Self {
        Self { state, reactor }
    }

    /// Updates the state via [`<R as Reducer<A>>::reduce`](trait.Reducer.html#tymethod.reduce) and
    /// notifies the reactor, returning the result of calling
    /// [`<S as Reactor<R>>::react`](trait.Reactor.html#tymethod.react) with a reference to the
    /// new state.
    pub fn dispatch<A>(&mut self, action: A) -> S::Output
    where
        R: Reducer<A>,
    {
        self.state.reduce(action);
        self.reactor.react(&self.state)
    }

    /// Replaces the reactor and returns the previous one.
    pub fn subscribe<T>(&mut self, reactor: T) -> S
    where
        T: Into<S>,
    {
        mem::replace(&mut self.reactor, reactor.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock::*;

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

        let a = NotSync::new(5);
        assert_eq!(store.dispatch(a), MockReducer::new(vec![5]));

        let a = NotSync::new(1);
        assert_eq!(store.dispatch(a), MockReducer::new(vec![5, 1]));

        let a = NotSyncOrSend::new(3);
        assert_eq!(store.dispatch(a), MockReducer::new(vec![5, 1, 3]));
    }

    #[test]
    fn subscribe() {
        let mut store: Store<_, Option<MockReactor<_>>> = Store::new(MockReducer::default(), None);

        assert_eq!(store.dispatch(0), None);

        store.subscribe(Some(MockReactor::default()));

        let a = NotSync::new(5);
        assert_eq!(store.dispatch(a), Some(MockReducer::new(vec![0, 5])));

        let a = NotSync::new(1);
        assert_eq!(store.dispatch(a), Some(MockReducer::new(vec![0, 5, 1])));

        let a = NotSyncOrSend::new(3);
        assert_eq!(store.dispatch(a), Some(MockReducer::new(vec![0, 5, 1, 3])));
    }
}
