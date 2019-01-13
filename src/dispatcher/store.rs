use dispatcher::Dispatcher;
use reactor::Reactor;
use reducer::Reducer;
use std::mem;

/// A reactive state container that manages the state of your application.
///
/// The only way to mutate the internal state managed by Store is by
/// [dispatching](trait.Dispatcher.html) actions on it.
/// The associated reactor is notified upon every state transition.
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
