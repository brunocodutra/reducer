use reducer::Reducer;
use std::mem;
use subscriber::Subscriber;

/// A reactive state container that manages the state of your application.
///
/// Store promotes reactive programming by encapsulating the state of your application and
/// notifying a [subscriber](struct.Store.html#method.subscribe) upon every change.
///
/// The only way to mutate the internal state managed by Store is by
/// [dispatching](struct.Store.html#method.dispatch) actions on it.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Store<R: Reducer, S: Subscriber<R>> {
    state: R,
    subscriber: S,
}

impl<R: Reducer, S: Subscriber<R>> Store<R, S> {
    /// Constructs the store given the initial state and a subscriber.
    pub fn new(state: R, subscriber: S) -> Self {
        Self { state, subscriber }
    }

    /// Updates the state via [`<R as Reducer>::reduce`](trait.Reducer.html#method.reduce) and
    /// notifies the subscriber, returning the result of calling
    /// [`<S as Subscriber<R>>::notify`](trait.Reducer.html#method.reduce) with a reference to the
    /// new state.
    pub fn dispatch(&mut self, action: impl Into<R::Action>) -> Result<(), S::Error> {
        self.state.reduce(action.into());
        self.subscriber.notify(&self.state)
    }

    /// Replaces the subscriber and returns the previous one.
    pub fn subscribe(&mut self, subscriber: impl Into<S>) -> S {
        mem::replace(&mut self.subscriber, subscriber.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reducer::MockReducer;
    use subscriber::MockSubscriber;

    #[test]
    fn default() {
        let store = Store::<MockReducer<()>, MockSubscriber<_>>::default();

        assert_eq!(store.state, MockReducer::default());
        assert_eq!(store.subscriber, MockSubscriber::default());
    }

    #[test]
    fn new() {
        let state = MockReducer::new(vec![42]);
        let subscriber = MockSubscriber::new(vec![state.clone()]);
        let store = Store::new(state.clone(), &subscriber);

        assert_eq!(store.state, state);
        assert_eq!(store.subscriber, &subscriber);
    }

    #[test]
    fn clone() {
        let store = Store::new(MockReducer::<()>::default(), MockSubscriber::default());
        assert_eq!(store, store.clone());
    }

    #[test]
    fn dispatch() {
        let mut store = Store::<MockReducer<_>, MockSubscriber<_>>::default();

        assert!(store.dispatch(5).is_ok());
        assert!(store.dispatch(1).is_ok());
        assert!(store.dispatch(3).is_ok());

        store.subscriber.set_result(Err);
        assert!(store.dispatch(false).is_err());
        store.subscriber.set_result(Ok);

        assert_eq!(store.state, MockReducer::new(vec![5, 1, 3, false.into()]));

        assert_eq!(
            store.subscriber,
            MockSubscriber::new(vec![
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

        store.subscribe(Some(MockSubscriber::default()));

        assert!(store.dispatch(5).is_ok());
        assert!(store.dispatch(1).is_ok());
        assert!(store.dispatch(3).is_ok());

        assert_eq!(store.state, MockReducer::new(vec![0, 5, 1, 3]));

        assert_eq!(
            store.subscriber,
            Some(MockSubscriber::new(vec![
                MockReducer::new(vec![0, 5]),
                MockReducer::new(vec![0, 5, 1]),
                MockReducer::new(vec![0, 5, 1, 3]),
            ])),
        );
    }
}
