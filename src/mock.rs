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

    #[test]
    fn react() {
        let reactor = MockReactor::default();

        assert_eq!(reactor.react(&5), 5);
        assert_eq!(reactor.react(&1), 1);
        assert_eq!(reactor.react(&3), 3);
    }

    #[test]
    fn reduce() {
        let mut state = MockReducer::default();

        state.reduce(5);
        state.reduce(1);
        state.reduce(3);

        assert_eq!(state, MockReducer::new(vec![5, 1, 3]));
    }

    #[test]
    fn dispatch() {
        let mut dispatcher = MockDispatcher::default();

        assert_eq!(dispatcher.dispatch(5), 5);
        assert_eq!(dispatcher.dispatch(1), 1);
        assert_eq!(dispatcher.dispatch(3), 3);
    }
}
