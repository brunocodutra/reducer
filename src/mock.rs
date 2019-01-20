#![cfg(test)]

use crate::dispatcher::Dispatcher;
use crate::reactor::Reactor;
use crate::reducer::Reducer;
use std::marker::PhantomData;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct MockReducer<A: 'static> {
    actions: Vec<A>,
}

impl<A> MockReducer<A> {
    pub fn new(actions: Vec<A>) -> Self {
        Self { actions }
    }
}

impl<A> Reducer<A> for MockReducer<A> {
    fn reduce(&mut self, action: A) {
        self.actions.push(action);
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct MockReactor<S>(PhantomData<S>);

impl<S: Clone> Reactor<S> for MockReactor<S> {
    type Output = S;

    fn react(&self, state: &S) -> Self::Output {
        state.clone()
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct MockDispatcher<A>(PhantomData<A>);

impl<A> Dispatcher<A> for MockDispatcher<A> {
    type Output = A;

    fn dispatch(&mut self, state: A) -> Self::Output {
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::*;

    proptest! {
        #[test]
        fn reduce(actions: Vec<u8>) {
            let mut reducer = MockReducer::default();

            for &action in &actions {
                reducer.reduce(action);
            }

            assert_eq!(reducer, MockReducer::new(actions));
        }
    }

    proptest! {
        #[test]
        fn react(states: Vec<u8>) {
            let reactor = MockReactor::default();

            for state in states {
                assert_eq!(reactor.react(&state), state);
            }
        }
    }

    proptest! {
        #[test]
        fn dispatch(actions: Vec<u8>) {
            let mut dispatcher = MockDispatcher::default();

            for action in actions {
                assert_eq!(dispatcher.dispatch(action), action);
            }
        }
    }
}
