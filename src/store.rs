use reducer::Reducer;
use subscriber::Subscriber;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Store<R: Reducer, S: Subscriber<R>> {
    state: R,
    subscriber: S,
}

impl<R: Reducer, S: Subscriber<R>> Store<R, S> {
    pub fn new(state: R, subscriber: S) -> Self {
        Self { state, subscriber }
    }

    pub fn dispatch(&mut self, action: impl Into<R::Action>) -> Result<(), S::Error> {
        self.state.reduce(action.into());
        self.subscriber.notify(&self.state)
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
}
