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
}
